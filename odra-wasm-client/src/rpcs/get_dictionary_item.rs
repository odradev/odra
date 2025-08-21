use casper_client::rpcs::DictionaryItemIdentifier;
use casper_types::{bytesrepr::Bytes, CLTyped, Key};
use odra_core::prelude::Address;

use crate::OdraWasmClient;

impl OdraWasmClient {
    pub(crate) async fn query_dict(
        &self,
        address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<Bytes, String> {
        let entity_addr = self.query_global_state_for_entity_addr(address).await?;
        let key = Key::Hash(entity_addr.value()).to_formatted_string();
        let identifier = DictionaryItemIdentifier::ContractNamedKey {
            key,
            dictionary_name,
            dictionary_item_key
        };

        let state_root_hash = self
            .get_state_root_hash()
            .await
            .map_err(|err| format!("Error getting state root hash: {err:?}"))?
            .ok_or(format!("State root hash is None, cannot get balance"))?;

        let stored_value = casper_client::get_dictionary_item(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            state_root_hash,
            identifier
        )
        .await
        .map(|response| response.result.stored_value)
        .map_err(|e| format!("Error getting dictionary item: {e}"))?;

        let cl_value = stored_value
            .into_cl_value()
            .ok_or(format!("Error converting stored value to CLValue"))?;

        // Note: this is for compatibility with CEP18 named keys.
        if cl_value.cl_type() == &Vec::<u8>::cl_type() {
            let bytes = cl_value
                .into_t()
                .map_err(|_| format!("Error converting CLValue to bytes"))?;
            Ok(bytes)
        } else {
            let bytes = cl_value.inner_bytes();
            Ok(Bytes::from(bytes.to_vec()))
        }
    }
}
