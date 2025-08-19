use casper_client::{
    cli::get_state_root_hash as get_state_root_hash_cli,
    rpcs::results::GetStateRootHashResult as _GetStateRootHashResult,
};
use rand::random;
// #[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

use crate::{types::digest::Digest, OdraWasmClient};

// #[cfg(target_arch = "wasm32")]
#[derive(Debug, Deserialize, Clone, Serialize)]
#[wasm_bindgen]
pub struct GetStateRootHashResult(_GetStateRootHashResult);

// #[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl GetStateRootHashResult {
    /// Gets the state root hash as an Option<Digest>.
    #[wasm_bindgen(getter)]
    pub fn state_root_hash(&self) -> Option<Digest> {
        self.0.state_root_hash.map(Into::into)
    }

    /// Gets the state root hash as a String.
    #[wasm_bindgen(getter)]
    pub fn state_root_hash_as_string(&self) -> String {
        self.0
            .state_root_hash
            .map(Into::<Digest>::into)
            .map(|digest| digest.to_string())
            .unwrap_or_default()
    }

    /// Alias for state_root_hash_as_string
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        // You can still use to_string method for compatibility
        self.state_root_hash_as_string()
    }
}

#[wasm_bindgen]
impl OdraWasmClient {
    #[wasm_bindgen(js_name = "get_state_root_hash")]
    pub async fn get_state_root_hash_js_alias(
        &self
    ) -> Result<GetStateRootHashResult, JsError> {
        let random_id = random::<u32>();
        get_state_root_hash_cli(&random_id.to_string(), self.node_address(), self.verbosity().into(), "")
            .await
            .map(|r| GetStateRootHashResult(r.result))
            .map_err(|_| {
                JsError::new(&format!(
                    "Couldn't get state root hash from node: {:?}",
                    self.node_address()
                ))
            })
    }
}
