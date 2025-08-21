use serde::{Deserialize, Serialize};
use std::ops::Deref;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct U256(casper_types::U256);

#[wasm_bindgen]
impl U256 {
    #[wasm_bindgen(constructor)]
    pub fn from_dec_str(value: &str) -> Self {
        U256(casper_types::U256::from_dec_str(value).unwrap())
    }
}

impl Deref for U256 {
    type Target = casper_types::U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<U256> for casper_types::U256 {
    fn from(value: U256) -> Self {
        value.0
    }
}

impl From<casper_types::U256> for U256 {
    fn from(value: casper_types::U256) -> Self {
        U256(value)
    }
}
