use crate::types::verbosity::Verbosity;
use casper_client::JsonRpcId;
use wasm_bindgen::prelude::*;

mod rpcs;
mod types;

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

    fn rpc_id_typed(&self) -> JsonRpcId {
        JsonRpcId::String("1".to_string())
    }
}
