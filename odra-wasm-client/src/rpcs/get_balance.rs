use crate::{types::address::Address, OdraWasmClient};
use casper_client::rpcs::results::GetBalanceResult as _GetBalanceResult;
use casper_types::U512;
#[cfg(target_arch = "wasm32")]
use gloo_utils::format::JsValueSerdeExt;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsError;

// Define a struct to wrap the GetBalanceResult
#[derive(Debug, Deserialize, Clone, Serialize)]
#[wasm_bindgen]
pub struct GetBalanceResult(_GetBalanceResult);

#[cfg(target_arch = "wasm32")]
impl From<GetBalanceResult> for _GetBalanceResult {
    fn from(result: GetBalanceResult) -> Self {
        result.0
    }
}

#[cfg(target_arch = "wasm32")]
impl From<_GetBalanceResult> for GetBalanceResult {
    fn from(result: _GetBalanceResult) -> Self {
        GetBalanceResult(result)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl GetBalanceResult {
    /// Gets the balance value as a JsValue.
    #[wasm_bindgen(getter)]
    pub fn balance_value(&self) -> JsValue {
        JsValue::from_serde(&self.0.balance_value).unwrap()
    }

    /// Converts the GetBalanceResult to a JsValue.
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(&self.0).unwrap_or(JsValue::null())
    }
}

impl GetBalanceResult {
    pub fn val(&self) -> U512 {
        self.0.balance_value
    }
}

#[wasm_bindgen]
impl OdraWasmClient {
    #[wasm_bindgen(js_name = "getBalance")]
    pub async fn get_balance(&self, address: Address) -> Result<GetBalanceResult, JsError> {
        let state_root_hash = self
            .get_state_root_hash()
            .await
            .map_err(|err| JsError::new(&format!("Error getting state root hash: {err:?}")))?
            .ok_or(JsError::new("State root hash is None, cannot get balance"))?;

        let purse = self
            .get_main_purse(&address)
            .await
            .map_err(|err| JsError::new(&err))?;

        let result = casper_client::get_balance(
            self.rpc_id(),
            &self.node_address,
            self.verbosity().into(),
            state_root_hash,
            purse
        )
        .await
        .map_err(JsError::from);

        match result {
            Ok(data) => Ok(data.result.into()),
            Err(err) => {
                let err = &format!("Error occurred with {err:?}");
                Err(JsError::new(err))
            }
        }
    }
}
