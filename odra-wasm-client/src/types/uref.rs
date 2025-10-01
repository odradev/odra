// use crate::types::{uref_addr::URefAddr};
use casper_types::{AccessRights, URef as _URef, URefAddr, UREF_ADDR_LENGTH};
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Deserialize, Clone, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct URef(_URef);

impl URef {
    pub fn new(uref_hex_str: &str, access_rights: u8) -> Result<Self, String> {
        // Convert the input hexadecimal string to bytes
        let bytes = match hex::decode(uref_hex_str) {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(format!("Invalid hex string: {err:?}"));
            }
        };

        if bytes.len() != UREF_ADDR_LENGTH {
            return Err(String::from("Invalid URefAddr length"));
        }
        let mut array = [0u8; UREF_ADDR_LENGTH];
        array.copy_from_slice(&bytes);
        let uref_addr = URefAddr::from(array);

        let uref = _URef::new(
            uref_addr,
            AccessRights::from_bits(access_rights).unwrap_or_default()
        );

        Ok(URef(uref))
    }

    pub fn from_formatted_str(formatted_str: &str) -> Result<Self, String> {
        let uref = _URef::from_formatted_str(formatted_str)
            .map_err(|error| format!("Failed to parse URef from formatted string: {error:?}"))?;
        Ok(URef(uref))
    }
}

#[wasm_bindgen]
impl URef {
    #[wasm_bindgen(constructor)]
    pub fn new_js_alias(uref_hex_str: &str, access_rights: u8) -> Result<URef, JsError> {
        Self::new(uref_hex_str, access_rights)
            .map_err(|err| JsError::new(&format!("Failed to parse URef from hex string: {err:?}")))
    }

    #[wasm_bindgen(js_name = "fromFormattedStr")]
    pub fn from_formatted_str_js_alias(formatted_str: &str) -> Result<URef, JsError> {
        Self::from_formatted_str(formatted_str).map_err(|err| {
            JsError::new(&format!(
                "Failed to parse URef from formatted string: {err:?}"
            ))
        })
    }

    #[wasm_bindgen(js_name = "fromUint8Array")]
    pub fn from_bytes(bytes: Vec<u8>, access_rights: u8) -> Self {
        let mut address_array = [0u8; 32];
        address_array[..bytes.len()].copy_from_slice(&bytes);

        URef(_URef::new(
            address_array,
            AccessRights::from_bits(access_rights).unwrap_or_default()
        ))
    }

    #[wasm_bindgen(js_name = "toFormattedString")]
    pub fn to_formatted_string(&self) -> String {
        self.0.to_formatted_string()
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
}

impl From<_URef> for URef {
    fn from(uref: _URef) -> Self {
        URef(uref)
    }
}

impl From<URef> for _URef {
    fn from(uref: URef) -> Self {
        uref.0
    }
}

impl Deref for URef {
    type Target = _URef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
