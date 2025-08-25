use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    PublicKey as _PublicKey, ED25519_TAG, SECP256K1_TAG, SYSTEM_TAG
};
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Deref
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicKey(_PublicKey);

impl PublicKey {
    pub fn new(public_key_hex_str: &str) -> Result<PublicKey, String> {
        let bytes = match hex::decode(public_key_hex_str) {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(format!("Failed to decode hex string: {err:?}"));
            }
        };

        // Convert the bytes to the public key
        let (public_key, _) = match _PublicKey::from_bytes(&bytes) {
            Ok(result) => result,
            Err(err) => {
                return Err(format!("Failed to parse public key bytes: {err:?}"));
            }
        };

        Ok(PublicKey(public_key))
    }

    pub fn tag(&self) -> u8 {
        match self.0.clone() {
            _PublicKey::System => SYSTEM_TAG,
            _PublicKey::Ed25519(_) => ED25519_TAG,
            _PublicKey::Secp256k1(_) => SECP256K1_TAG,
            _ => unimplemented!()
        }
    }
}

impl Deref for PublicKey {
    type Target = _PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[wasm_bindgen]
impl PublicKey {
    #[wasm_bindgen(constructor)]
    pub fn new_js_alias(public_key_hex_str: &str) -> Result<PublicKey, JsError> {
        Self::new(public_key_hex_str).map_err(|err| {
            JsError::new(&format!(
                "Failed to parse PublicKey from hex string: {err:?}"
            ))
        })
    }

    #[wasm_bindgen(js_name = "fromUint8Array")]
    pub fn from_bytes_js_alias(bytes: Vec<u8>) -> Result<PublicKey, JsError> {
        Self::from_bytes(&bytes)
            .map(|(public_key, _)| public_key)
            .map_err(|err| JsError::new(&format!("Failed to parse PublicKey: {err:?}")))
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let bytes = self.0.to_bytes().unwrap_or_default();
        let hex_string = hex::encode(bytes);
        write!(f, "{hex_string}")
    }
}

impl From<PublicKey> for _PublicKey {
    fn from(public_key: PublicKey) -> Self {
        public_key.0
    }
}

impl From<_PublicKey> for PublicKey {
    fn from(public_key: _PublicKey) -> Self {
        PublicKey(public_key)
    }
}

impl ToBytes for PublicKey {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }

    fn write_bytes(&self, bytes: &mut Vec<u8>) -> Result<(), bytesrepr::Error> {
        self.0.write_bytes(bytes)
    }
}

impl FromBytes for PublicKey {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (public_key, remainder) = _PublicKey::from_bytes(bytes)?;
        Ok((PublicKey(public_key), remainder))
    }
}
