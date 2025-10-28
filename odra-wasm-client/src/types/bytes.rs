use casper_types::{bytesrepr::Bytes as _Bytes, CLType, CLTyped};
use core::ops::Deref;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default, Hash)]
pub struct Bytes(Vec<u8>);

#[wasm_bindgen]
impl Bytes {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Bytes(Vec::new())
    }

    #[wasm_bindgen(js_name = "fromUint8Array")]
    pub fn from_uint8_array(uint8_array: js_sys::Uint8Array) -> Self {
        let length = uint8_array.length() as usize;
        let mut bytes_vec = Vec::with_capacity(length);

        for i in 0..length {
            bytes_vec.push(uint8_array.get_index(i.try_into().unwrap()));
        }
        Self::from(bytes_vec)
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn stringify(&self) -> String {
        hex::encode(&self.0)
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Bytes(vec)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Bytes(bytes.to_vec())
    }
}

impl CLTyped for Bytes {
    fn cl_type() -> CLType {
        <Vec<u8>>::cl_type()
    }
}

impl From<Bytes> for _Bytes {
    fn from(bytes: Bytes) -> Self {
        _Bytes::from(bytes.0)
    }
}

impl From<_Bytes> for Bytes {
    fn from(bytes: _Bytes) -> Self {
        Bytes(bytes.into())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
