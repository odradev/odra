use casper_client::rpcs::GlobalStateIdentifier;
use casper_types::{Key, StoredValue};

use crate::OdraWasmClient;

impl OdraWasmClient {
    pub(crate) async fn query_global_state(
        &self,
        key: Key,
        path: Option<String>
    ) -> Option<StoredValue> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let digest = self.get_state_root_hash().await.ok().flatten()?;
        let result = casper_client::query_global_state(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            GlobalStateIdentifier::StateRootHash(digest),
            key,
            path
        )
        .await;
        match result {
            Ok(r) => Some(r.result.stored_value),
            Err(_) => None
        }
    }
}
