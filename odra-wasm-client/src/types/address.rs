use std::ops::Deref;

use odra_core::prelude::Address as _Address;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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