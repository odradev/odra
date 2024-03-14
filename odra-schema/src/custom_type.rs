use std::collections::BTreeMap;

use casper_contract_schema::CustomType;
use casper_types::{bytesrepr::Bytes, CLTyped, PublicKey, U128, U256, U512};
use odra_core::{args::Maybe, Address};

pub trait SchemaCustomType {
    fn custom_ty() -> Option<CustomType> {
        None
    }
}

impl SchemaCustomType for u8 {}

impl SchemaCustomType for u32 {}

impl SchemaCustomType for u64 {}

impl SchemaCustomType for i32 {}

impl SchemaCustomType for i64 {}

impl SchemaCustomType for U128 {}

impl SchemaCustomType for U256 {}

impl SchemaCustomType for U512 {}

impl SchemaCustomType for Address {}

impl SchemaCustomType for bool {}

impl SchemaCustomType for String {}

impl SchemaCustomType for () {}

impl SchemaCustomType for PublicKey {}

impl SchemaCustomType for Bytes {}

impl<T: CLTyped> SchemaCustomType for Option<T> {}

impl<T: CLTyped> SchemaCustomType for Vec<T> {}

impl<T: CLTyped, E: CLTyped> SchemaCustomType for Result<T, E> {}

impl<T: CLTyped, E: CLTyped> SchemaCustomType for BTreeMap<T, E> {}

impl<T1: CLTyped> SchemaCustomType for (T1,) {}

impl<T1: CLTyped, T2: CLTyped> SchemaCustomType for (T1, T2) {}

impl<T1: CLTyped, T2: CLTyped, T3: CLTyped> SchemaCustomType for (T1, T2, T3) {}

impl<const COUNT: usize> SchemaCustomType for [u8; COUNT] {}

impl<T: SchemaCustomType> SchemaCustomType for Maybe<T> {
    fn custom_ty() -> Option<CustomType> {
        T::custom_ty()
    }
}
