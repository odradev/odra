use crate::OdraWasmClient;
use casper_types::Digest;

impl OdraWasmClient {
    pub(crate) async fn get_state_root_hash(&self) -> Result<Option<Digest>, String> {
        casper_client::get_state_root_hash(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            None
        )
        .await
        .map(|r| r.result.state_root_hash)
        .map_err(|_| {
            format!(
                "Couldn't get state root hash from node: {:?}",
                self.node_address()
            )
        })
    }
}
