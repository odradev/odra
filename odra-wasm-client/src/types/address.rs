use std::ops::Deref;

use odra_core::prelude::Address as _Address;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::types::public_key::PublicKey;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[wasm_bindgen]
pub struct Address(_Address);

#[wasm_bindgen]
impl Address {
    #[wasm_bindgen(constructor)]
    pub fn new(address: &str) -> Result<Address, JsError> {
        use std::str::FromStr;

        _Address::from_str(address)
            .map(Address)
            .map_err(|err| JsError::new(&format!("{err:?}")))
    }
}

impl Deref for Address {
    type Target = _Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Address> for _Address {
    fn from(address: Address) -> Self {
        address.0
    }
}

impl From<PublicKey> for Address {
    fn from(value: PublicKey) -> Self {
        let pk: casper_types::PublicKey = value.into();
        Address(_Address::from(pk))
    }
}

impl From<_Address> for Address {
    fn from(address: _Address) -> Self {
        Address(address)
    }
}
