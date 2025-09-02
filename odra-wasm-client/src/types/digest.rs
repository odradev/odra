use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    Digest as _Digest, DigestError
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[wasm_bindgen]
pub struct Digest(_Digest);

impl Digest {
    pub fn value(&self) -> [u8; _Digest::LENGTH] {
        self.0.value()
    }

    pub fn from_raw(bytes: Vec<u8>) -> Result<Digest, String> {
        let hex_string = hex::encode(bytes);
        Digest::try_from(&hex_string[..]).map_err(|e| e.to_string())
    }
}

#[wasm_bindgen]
impl Digest {
    #[wasm_bindgen(constructor)]
    pub fn new_js_alias(digest_hex_str: &str) -> Result<Digest, JsError> {
        Self::from_string(digest_hex_str)
    }

    #[wasm_bindgen(js_name = "fromString")]
    pub fn from_string(digest_hex_str: &str) -> Result<Digest, JsError> {
        Self::try_from(digest_hex_str).map_err(|err| JsError::new(&format!("{err:?}")))
    }

    #[wasm_bindgen(js_name = "fromRaw")]
    pub fn from_raw_js_alias(bytes: Vec<u8>) -> Result<Digest, JsError> {
        Self::from_raw(bytes).map_err(|err| JsError::new(&format!("{err:?}")))
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        self.to_string()
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<Digest> for _Digest {
    fn from(digest: Digest) -> Self {
        digest.0
    }
}

impl From<_Digest> for Digest {
    fn from(digest: _Digest) -> Self {
        Digest(digest)
    }
}

impl ToBytes for Digest {
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

impl FromBytes for Digest {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        _Digest::from_bytes(bytes).map(|(digest, remainder)| (Digest(digest), remainder))
    }
}

impl From<[u8; _Digest::LENGTH]> for Digest {
    fn from(bytes: [u8; _Digest::LENGTH]) -> Self {
        let digest = _Digest::from(bytes);
        Self(digest)
    }
}

impl TryFrom<&str> for Digest {
    type Error = String;
    fn try_from(digest_hex_str: &str) -> Result<Self, Self::Error> {
        let bytes = hex::decode(digest_hex_str)
            .map_err(|err| format!("Failed to decode hex string {digest_hex_str:?}: {err:?}"))?;

        if bytes.len() != _Digest::LENGTH {
            let context = "Invalid Digest length".to_string();
            let digest_error = DigestError::IncorrectDigestLength(bytes.len());
            return Err(format!("Failed to parse digest: {context}, {digest_error}"));
        }

        let mut digest_bytes = [0u8; _Digest::LENGTH];
        digest_bytes.copy_from_slice(&bytes);
        Ok(Self(_Digest::from(digest_bytes)))
    }
}

#[allow(unused)]
pub trait ToDigest {
    fn to_digest(&self) -> Digest;
    fn is_empty(&self) -> bool;
}

impl ToDigest for Digest {
    fn to_digest(&self) -> Digest {
        self.0.into()
    }
    fn is_empty(&self) -> bool {
        hex::encode(self.0).is_empty()
    }
}

impl ToDigest for &str {
    fn to_digest(&self) -> Digest {
        Digest::try_from(*self).unwrap_or_default()
    }

    fn is_empty(&self) -> bool {
        self.trim().is_empty()
    }
}
