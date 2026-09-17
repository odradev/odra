//! Client for interacting with Casper node.

use crate::casper_client::{
    configuration::CasperClientConfiguration, transaction_watcher::TransactionWatcher
};
use crate::error::LivenetError;
use crate::log;
use casper_types::bytesrepr::Bytes;
use casper_types::{Digest, Key, StoredValue, U512};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub mod accounts;
pub mod configuration;
mod node;
mod queries;
pub mod transaction_watcher;
mod transactions;
mod validators;

/// Environment variable holding a path to a secret key of a main account.
pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
/// Environment variable holding an address of the casper node exposing RPC API.
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
/// Environment variable holding the URL of the events stream.
pub const ENV_EVENTS_ADDRESS: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
/// Environment variable holding a name of the chain.
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
/// Environment variable holding a filename prefix for additional accounts.
pub const ENV_ACCOUNT_PREFIX: &str = "ODRA_CASPER_LIVENET_KEY_";
/// Environment variable holding cspr.cloud auth token.
pub const ENV_CSPR_CLOUD_AUTH_TOKEN: &str = "CSPR_CLOUD_AUTH_TOKEN";
/// Environment variable holding a path to an additional .env file.
pub const ENV_LIVENET_ENV_FILE: &str = "ODRA_CASPER_LIVENET_ENV";
/// Environment variable holding TTL for transactions.
pub const ENV_TTL: &str = "ODRA_CASPER_LIVENET_TTL";
/// Environment variable holding gas price tolerance for transactions.
pub const ENV_GAS_PRICE_TOLERANCE: &str = "ODRA_CASPER_LIVENET_GAS_PRICE_TOLERANCE";
/// Environment variable pinning every read to a past state root hash (hex). Transactions are
/// refused while it is set.
pub const ENV_STATE_ROOT_HASH: &str = "ODRA_CASPER_LIVENET_STATE_ROOT_HASH";

pub type Result<T> = core::result::Result<T, LivenetError>;

const TRANSACTION_WAIT_TIME: u64 = 10;
const TRANSACTION_MAX_RETRIES: u64 = 12;
/// How long a fetched state root hash is reused for queries. Every transaction sent by this
/// client drops it earlier, so the client's own writes are always visible to its next read.
const STATE_ROOT_HASH_TTL: Duration = Duration::from_secs(5);

/// Client for interacting with Casper node.
///
/// Every network call comes in two flavours:
/// - `xxx_async`: an `async fn`, the actual implementation. Use it from async code; several of them
///   can run at once with `futures::future::join_all` or `tokio::join!` (the futures borrow the
///   client, so they run on one task, which is all the concurrency the network needs).
/// - `xxx`: a blocking wrapper that drives the async one with [`utils::block_on`](crate::utils::block_on)
///   on a process-wide Tokio runtime. This is what the livenet `HostEnv` uses. It also works inside
///   a multi-thread Tokio runtime; inside a current-thread runtime it panics, use the async flavour.
///
/// The client keeps no runtime of its own, so it can be created and used from any thread; the
/// livenet `HostEnv` builds one per thread when work runs concurrently.
pub struct CasperClient {
    pub configuration: CasperClientConfiguration,
    watcher: TransactionWatcher,
    active_account: usize,
    gas: U512,
    /// A state root hash every read is pinned to (`ODRA_CASPER_LIVENET_STATE_ROOT_HASH`).
    pinned_state_root_hash: Option<Digest>,
    /// Cached state root hash and the time it was fetched, see [STATE_ROOT_HASH_TTL].
    state_root_hash: RefCell<Option<(Digest, Instant)>>,
    /// Query responses, valid for one state root hash.
    query_cache: RefCell<QueryCache>
}

/// Responses of global state and dictionary queries, keyed by the state root hash they were
/// read at. A query at a fixed state root is deterministic, so serving it from here is the same
/// as asking the node again; the whole map is dropped whenever the state root hash the queries
/// use moves on (see [STATE_ROOT_HASH_TTL]).
#[derive(Default)]
struct QueryCache {
    state_root_hash: Option<Digest>,
    global_state: BTreeMap<(Key, Vec<String>), Option<StoredValue>>,
    dictionary: BTreeMap<(String, String, String), Option<Bytes>>
}

impl QueryCache {
    /// Drops everything read at another state root.
    fn reset_to(&mut self, state_root_hash: Digest) {
        if self.state_root_hash != Some(state_root_hash) {
            self.state_root_hash = Some(state_root_hash);
            self.global_state.clear();
            self.dictionary.clear();
        }
    }
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(configuration: CasperClientConfiguration) -> Self {
        let timeout = Duration::from_secs(TRANSACTION_WAIT_TIME * TRANSACTION_MAX_RETRIES);
        let watcher = TransactionWatcher::new(&configuration, timeout);
        if let Some(digest) = configuration.state_root_hash {
            log::info(format!(
                "Reads pinned to state root hash {}; transactions are disabled.",
                base16::encode_lower(&digest)
            ));
        }
        CasperClient {
            pinned_state_root_hash: configuration.state_root_hash,
            configuration,
            watcher,
            active_account: 0,
            gas: U512::zero(),
            state_root_hash: RefCell::new(None),
            query_cache: RefCell::new(QueryCache::default())
        }
    }

    /// Global state query response cached at `state_root_hash`, if any.
    pub(crate) fn cached_global_state(
        &self,
        state_root_hash: Digest,
        key: &Key,
        path: &[String]
    ) -> Option<Option<StoredValue>> {
        let mut cache = self.query_cache.borrow_mut();
        cache.reset_to(state_root_hash);
        cache.global_state.get(&(*key, path.to_vec())).cloned()
    }

    pub(crate) fn cache_global_state(
        &self,
        state_root_hash: Digest,
        key: Key,
        path: Vec<String>,
        value: Option<StoredValue>
    ) {
        let mut cache = self.query_cache.borrow_mut();
        cache.reset_to(state_root_hash);
        cache.global_state.insert((key, path), value);
    }

    /// Dictionary item response cached at `state_root_hash`, if any.
    pub(crate) fn cached_dictionary_item(
        &self,
        state_root_hash: Digest,
        item: &(String, String, String)
    ) -> Option<Option<Bytes>> {
        let mut cache = self.query_cache.borrow_mut();
        cache.reset_to(state_root_hash);
        cache.dictionary.get(item).cloned()
    }

    pub(crate) fn cache_dictionary_item(
        &self,
        state_root_hash: Digest,
        item: (String, String, String),
        value: Option<Bytes>
    ) {
        let mut cache = self.query_cache.borrow_mut();
        cache.reset_to(state_root_hash);
        cache.dictionary.insert(item, value);
    }

    /// The state root hash all reads are pinned to, if any.
    pub fn pinned_state_root_hash(&self) -> Option<Digest> {
        self.pinned_state_root_hash
    }

    /// Fails when reads are pinned to a past state root hash: a transaction would execute at the
    /// chain tip and its effects would never show up in the pinned view.
    fn ensure_not_pinned(&self) -> Result<()> {
        match self.pinned_state_root_hash {
            None => Ok(()),
            Some(digest) => Err(LivenetError::ClientError(format!(
                "Transactions are disabled while reads are pinned to state root hash {} \
                 ({ENV_STATE_ROOT_HASH})",
                base16::encode_lower(&digest)
            )))
        }
    }

    /// Returns the pinned state root hash, or the cached one if it is younger than
    /// [STATE_ROOT_HASH_TTL].
    fn cached_state_root_hash(&self) -> Option<Digest> {
        if self.pinned_state_root_hash.is_some() {
            return self.pinned_state_root_hash;
        }
        self.state_root_hash
            .borrow()
            .filter(|(_, fetched_at)| fetched_at.elapsed() < STATE_ROOT_HASH_TTL)
            .map(|(digest, _)| digest)
    }

    fn cache_state_root_hash(&self, digest: Digest) {
        *self.state_root_hash.borrow_mut() = Some((digest, Instant::now()));
    }

    /// Forgets the cached state root hash; called after every transaction this client sends.
    /// A pinned state root hash stays.
    pub fn invalidate_state_root_hash(&self) {
        *self.state_root_hash.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::casper_client::configuration::CasperClientConfiguration;

    fn configuration(state_root_hash: Option<Digest>) -> CasperClientConfiguration {
        CasperClientConfiguration {
            node_address: "http://localhost:11101".to_string(),
            events_url: "http://localhost:18101/events".to_string(),
            chain_name: "casper-net-1".to_string(),
            secret_keys: vec![],
            secret_key_paths: vec![],
            cspr_cloud_auth_token: None,
            gas_price_tolerance: 1,
            ttl: 300,
            state_root_hash
        }
    }

    #[test]
    fn pinned_state_root_hash_is_used_and_survives_invalidation() {
        let digest = Digest::hash(b"past");
        let client = CasperClient::new(configuration(Some(digest)));
        assert_eq!(client.cached_state_root_hash(), Some(digest));
        client.invalidate_state_root_hash();
        assert_eq!(client.cached_state_root_hash(), Some(digest));
        assert!(client.ensure_not_pinned().is_err());
    }

    #[test]
    fn unpinned_client_starts_without_a_cached_hash() {
        let client = CasperClient::new(configuration(None));
        assert_eq!(client.cached_state_root_hash(), None);
        assert!(client.ensure_not_pinned().is_ok());
        client.cache_state_root_hash(Digest::hash(b"now"));
        assert!(client.cached_state_root_hash().is_some());
        client.invalidate_state_root_hash();
        assert_eq!(client.cached_state_root_hash(), None);
    }
}
