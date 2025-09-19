use std::collections::BTreeMap;

use gloo_utils::format::JsValueSerdeExt;
use serde::Serialize;
use wasm_bindgen::{JsError, JsValue};

use crate::types::*;

pub trait IntoWasmValue<T> {
    fn to_wasm_value(self) -> T;
}

impl IntoWasmValue<PublicKey> for casper_types::PublicKey {
    fn to_wasm_value(self) -> PublicKey {
        PublicKey::from(self)
    }
}

impl IntoWasmValue<URef> for casper_types::URef {
    fn to_wasm_value(self) -> URef {
        URef::from(self)
    }
}

impl IntoWasmValue<Address> for odra_core::prelude::Address {
    fn to_wasm_value(self) -> Address {
        Address::from(self)
    }
}

impl IntoWasmValue<Bytes> for Vec<u8> {
    fn to_wasm_value(self) -> Bytes {
        Bytes::from(self)
    }
}

impl IntoWasmValue<U512> for casper_types::U512 {
    fn to_wasm_value(self) -> U512 {
        U512::from(self)
    }
}

impl IntoWasmValue<U256> for casper_types::U256 {
    fn to_wasm_value(self) -> U256 {
        U256::from(self)
    }
}
impl IntoWasmValue<U128> for casper_types::U128 {
    fn to_wasm_value(self) -> U128 {
        U128::from(self)
    }
}

impl IntoWasmValue<u8> for u8 {
    fn to_wasm_value(self) -> u8 {
        self
    }
}

impl IntoWasmValue<u32> for u32 {
    fn to_wasm_value(self) -> u32 {
        self
    }
}

impl IntoWasmValue<u64> for u64 {
    fn to_wasm_value(self) -> u64 {
        self
    }
}

impl IntoWasmValue<i32> for i32 {
    fn to_wasm_value(self) -> i32 {
        self
    }
}

impl IntoWasmValue<i64> for i64 {
    fn to_wasm_value(self) -> i64 {
        self
    }
}

impl IntoWasmValue<String> for String {
    fn to_wasm_value(self) -> String {
        self
    }
}

impl IntoWasmValue<bool> for bool {
    fn to_wasm_value(self) -> bool {
        self
    }
}

impl IntoWasmValue<()> for () {
    fn to_wasm_value(self) -> () {
        ()
    }
}

// impl<T, U> IntoWasmValue<Vec<U>> for Vec<Option<T>>
// where
//     T: IntoWasmValue<U>
// {
//     fn to_wasm_value(self) -> Vec<U> {
//         self.into_iter().map(IntoWasmValue::to_wasm_value).collect()
//     }
// }

impl<T, U> IntoWasmValue<Vec<U>> for Vec<T>
where
    T: IntoWasmValue<U>
{
    fn to_wasm_value(self) -> Vec<U> {
        self.into_iter().map(IntoWasmValue::to_wasm_value).collect()
    }
}

impl<T, U> IntoWasmValue<Option<U>> for Option<T>
where
    T: IntoWasmValue<U>
{
    fn to_wasm_value(self) -> Option<U> {
        self.map(IntoWasmValue::to_wasm_value)
    }
}

impl<T: Serialize> IntoWasmValue<JsValue> for Option<T> {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

impl<T: Serialize, U: Serialize> IntoWasmValue<JsValue> for BTreeMap<T, U> {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

impl<T: Serialize> IntoWasmValue<JsValue> for (T,) {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

impl<T: Serialize, U: Serialize> IntoWasmValue<JsValue> for (T, U) {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

impl<T: Serialize, U: Serialize, V: Serialize> IntoWasmValue<JsValue> for (T, U, V) {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

impl<T: Serialize, U: Serialize> IntoWasmValue<JsValue> for Result<T, U> {
    fn to_wasm_value(self) -> JsValue {
        JsValue::from_serde(&self).unwrap()
    }
}

pub trait FromWasmValue<T> {
    fn from_wasm_value(self) -> Result<T, JsError>;
}

impl FromWasmValue<casper_types::PublicKey> for PublicKey {
    fn from_wasm_value(self) -> Result<casper_types::PublicKey, JsError> {
        Ok(self.into())
    }
}

impl FromWasmValue<casper_types::PublicKey> for &PublicKey {
    fn from_wasm_value(self) -> Result<casper_types::PublicKey, JsError> {
        Ok(self.clone().into())
    }
}

impl FromWasmValue<casper_types::URef> for URef {
    fn from_wasm_value(self) -> Result<casper_types::URef, JsError> {
        Ok(self.into())
    }
}

impl FromWasmValue<casper_types::URef> for &URef {
    fn from_wasm_value(self) -> Result<casper_types::URef, JsError> {
        Ok(self.clone().into())
    }
}

impl FromWasmValue<odra_core::prelude::Address> for Address {
    fn from_wasm_value(self) -> Result<odra_core::prelude::Address, JsError> {
        Ok(self.into())
    }
}

impl FromWasmValue<Vec<u8>> for Bytes {
    fn from_wasm_value(self) -> Result<Vec<u8>, JsError> {
        Ok(self.into())
    }
}

impl FromWasmValue<casper_types::U512> for U512 {
    fn from_wasm_value(self) -> Result<casper_types::U512, JsError> {
        Ok(casper_types::U512::from(self))
    }
}

impl FromWasmValue<casper_types::U256> for U256 {
    fn from_wasm_value(self) -> Result<casper_types::U256, JsError> {
        Ok(casper_types::U256::from(self))
    }
}

impl FromWasmValue<casper_types::U128> for U128 {
    fn from_wasm_value(self) -> Result<casper_types::U128, JsError> {
        Ok(casper_types::U128::from(self))
    }
}

impl FromWasmValue<u8> for u8 {
    fn from_wasm_value(self) -> Result<u8, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<u32> for u32 {
    fn from_wasm_value(self) -> Result<u32, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<u64> for u64 {
    fn from_wasm_value(self) -> Result<u64, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<i32> for i32 {
    fn from_wasm_value(self) -> Result<i32, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<i64> for i64 {
    fn from_wasm_value(self) -> Result<i64, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<String> for String {
    fn from_wasm_value(self) -> Result<String, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<bool> for bool {
    fn from_wasm_value(self) -> Result<bool, JsError> {
        Ok(self)
    }
}

impl FromWasmValue<()> for () {
    fn from_wasm_value(self) -> Result<(), JsError> {
        Ok(())
    }
}

impl<T: FromWasmValue<U>, U> FromWasmValue<Option<U>> for Option<T> {
    fn from_wasm_value(self) -> Result<Option<U>, JsError> {
        match self {
            Some(value) => Ok(Some(value.from_wasm_value()?)),
            None => Ok(None)
        }
    }
}

impl<T: FromWasmValue<U>, U> FromWasmValue<Vec<U>> for Vec<T> {
    fn from_wasm_value(self) -> Result<Vec<U>, JsError> {
        self.into_iter()
            .map(|item| item.from_wasm_value())
            .collect::<Result<_, _>>()
    }
}

impl<T: for<'de> serde::Deserialize<'de> + Serialize> FromWasmValue<T> for JsValue {
    fn from_wasm_value(self) -> Result<T, JsError> {
        self.into_serde()
            .map_err(|err| JsError::new(&format!("{:?}", err)))
    }
}
