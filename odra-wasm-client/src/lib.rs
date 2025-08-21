use crate::types::verbosity::Verbosity;
use casper_client::JsonRpcId;
use casper_types::{bytesrepr::Bytes, Key, StoredValue};
use odra_core::prelude::Address;
use wasm_bindgen::prelude::*;

mod cep18;
pub mod js;
mod rpcs;
mod types;
mod wallet;

#[wasm_bindgen]
pub struct OdraWasmClient {
    node_address: String,
    verbosity: Verbosity
}

#[wasm_bindgen]
impl OdraWasmClient {
    #[wasm_bindgen(constructor)]
    pub fn new(node_address: String, verbosity: Verbosity) -> Self {
        OdraWasmClient {
            node_address,
            verbosity
        }
    }
}

impl OdraWasmClient {
    pub fn node_address(&self) -> &str {
        &self.node_address
    }

    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn rpc_id(&self) -> JsonRpcId {
        JsonRpcId::String("1".to_string())
    }
}

#[wasm_bindgen]
impl OdraWasmClient {
    /// Gets a value from a named key of an account or a contract
    #[wasm_bindgen(js_name = "getNamedValue")]
    pub async fn get_named_value(
        &self,
        address: &crate::types::address::Address,
        name: &str
    ) -> Option<crate::types::bytes::Bytes> {
        let entity_hash = self
            .query_global_state_for_entity_addr(address)
            .await
            .ok()?;
        let stored_value = self
            .query_global_state(Key::Hash(entity_hash.value()), Some(name.to_string()))
            .await;
        match stored_value {
            None => None,
            Some(value) => match value {
                StoredValue::CLValue(value) => Some(crate::types::bytes::Bytes::from(
                    value.inner_bytes().as_slice()
                )),
                _ => {
                    panic!(
                        "Couldn't get {} from {:?}, instead of CLValue got {:?}",
                        name,
                        address.to_formatted_string(),
                        value
                    )
                }
            }
        }
    }
}

impl OdraWasmClient {
    /// Gets a value from the Odra storage (`state` dictionary)
    pub async fn get_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.get_dictionary_value(address, "state", key).await
    }

    /// Gets a value from a named dictionary
    pub async fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        let key = String::from_utf8(key.to_vec())
            .unwrap_or_else(|_| panic!("Couldn't convert key to string: {:?}", key));
        self.query_dict(address, dictionary_name.to_string(), key)
            .await
            .ok()
    }
}
