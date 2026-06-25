//! Client for interacting with Casper node.

use crate::casper_client::{
    configuration::CasperClientConfiguration, transaction_watcher::TransactionWatcher
};
use crate::error::LivenetError;
use casper_types::U512;
use std::rc::Rc;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

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

pub type Result<T> = core::result::Result<T, LivenetError>;

const TRANSACTION_WAIT_TIME: u64 = 10;
const TRANSACTION_MAX_RETRIES: u64 = 12;

/// Client for interacting with Casper node.
///
/// The client exposes a synchronous public API. Internally each network call is
/// driven by a single Tokio runtime owned by the client (`runtime`), so callers
/// don't need to manage an executor themselves. The async methods (`*_async`)
/// remain the internal engine and must only ever be composed via `.await` from
/// other async methods — never through the sync wrappers, which would
/// `block_on` inside `block_on` and panic.
pub struct CasperClient {
    pub configuration: CasperClientConfiguration,
    watcher: TransactionWatcher,
    active_account: usize,
    gas: U512,
    runtime: Rc<Runtime>
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(configuration: CasperClientConfiguration) -> Self {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime");

        let timeout = Duration::from_secs(TRANSACTION_WAIT_TIME * TRANSACTION_MAX_RETRIES);
        let watcher = TransactionWatcher::new(&configuration, timeout);
        CasperClient {
            configuration,
            watcher,
            active_account: 0,
            gas: U512::zero(),
            runtime: Rc::new(runtime)
        }
    }

    /// Returns a handle to the client's Tokio runtime.
    ///
    /// Cloning the `Rc` first lets a sync wrapper call `rt.block_on(self.x_async())`
    /// without borrowing `self` twice.
    fn runtime(&self) -> Rc<Runtime> {
        self.runtime.clone()
    }
}
