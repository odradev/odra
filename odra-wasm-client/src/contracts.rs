use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;

use crate::types::Address;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct Contracts {
    #[serde(alias = "time")]
    last_updated: String,
    contracts: Vec<ContractInfo>
}

#[wasm_bindgen]
impl Contracts {
    #[wasm_bindgen(constructor)]
    pub fn new(js: JsValue) -> Result<Self, JsError> {
        js.into_serde::<Contracts>()
            .and_then(Ok)
            .map_err(|err| JsError::new(&format!("Could not parse Contracts from JSON: {err:?}")))
    }

    #[wasm_bindgen]
    pub fn get(&self, name: &str) -> Result<ContractInfo, JsError> {
        let contract = self
            .contracts
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| JsError::new(&format!("Contract with name {:?} not found", name)))?;
        Ok(contract.clone())
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen(getter_with_clone)]
pub struct ContractInfo {
    pub name: String,
    pub package_hash: String
}

#[wasm_bindgen]
impl ContractInfo {
    #[wasm_bindgen(getter)]
    pub fn address(&self) -> Address {
        Address::new(&self.package_hash).expect("Invalid package hash")
    }
}
