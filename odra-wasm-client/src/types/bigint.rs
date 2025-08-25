use js_sys::BigInt as JsBigInt;
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

    #[wasm_bindgen(js_name = "fromU32")]
    pub fn from_u32(value: u32) -> Self {
        U256(casper_types::U256::from(value))
    }

    #[wasm_bindgen(js_name = "fromBigInt")]
    pub fn from_js_big_int(value: JsBigInt) -> Self {
        let v = value
            .to_string(10)
            .map(|s| s.as_string())
            .unwrap_or_default()
            .unwrap_or_default();
        Self::from_dec_str(&v)
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
