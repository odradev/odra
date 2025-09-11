use gloo_utils::format::JsValueSerdeExt;
use js_sys::BigInt as JsBigInt;

use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use std::ops::Deref;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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

    #[wasm_bindgen(js_name = "fromHtmlInput")]
    pub fn from_input(input: HtmlInputElement) -> Self {
        let value = input.value();
        Self::from_dec_str(value.trim())
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

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct U512(casper_types::U512);

#[wasm_bindgen]
impl U512 {
    #[wasm_bindgen(constructor)]
    pub fn from_dec_str(value: &str) -> Self {
        U512(casper_types::U512::from_dec_str(value).unwrap())
    }

    #[wasm_bindgen(js_name = "fromU32")]
    pub fn from_u32(value: u32) -> Self {
        U512(casper_types::U512::from(value))
    }

    #[wasm_bindgen(js_name = "fromHtmlInput")]
    pub fn from_input(input: HtmlInputElement) -> Self {
        let value = input.value();
        Self::from_dec_str(value.trim())
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

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        self.0.to_string()
    }
}

impl Deref for U512 {
    type Target = casper_types::U512;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<U512> for casper_types::U512 {
    fn from(value: U512) -> Self {
        value.0
    }
}

impl From<casper_types::U512> for U512 {
    fn from(value: casper_types::U512) -> Self {
        U512(value)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct U128(casper_types::U128);

#[wasm_bindgen]
impl U128 {
    #[wasm_bindgen(constructor)]
    pub fn from_dec_str(value: &str) -> Self {
        U128(casper_types::U128::from_dec_str(value).unwrap())
    }

    #[wasm_bindgen(js_name = "fromU32")]
    pub fn from_u32(value: u32) -> Self {
        U128(casper_types::U128::from(value))
    }

    #[wasm_bindgen(js_name = "fromHtmlInput")]
    pub fn from_input(input: HtmlInputElement) -> Self {
        let value = input.value();
        Self::from_dec_str(value.trim())
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

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        self.0.to_string()
    }
}

impl Deref for U128 {
    type Target = casper_types::U128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<U128> for casper_types::U128 {
    fn from(value: U128) -> Self {
        value.0
    }
}

impl From<casper_types::U128> for U128 {
    fn from(value: casper_types::U128) -> Self {
        U128(value)
    }
}
