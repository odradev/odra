use std::collections::BTreeMap;

use casper_contract_schema::CustomType;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::URef;
use casper_types::{bytesrepr::Bytes, CLTyped, PublicKey, U128, U256, U512};
use num_traits::{Num, One, Zero};
use odra_core::{args::Maybe, Address};
use odra_core::{List, Mapping, Sequence, SubModule, Var};

use crate::{SchemaCustomTypes, SchemaErrors, SchemaEvents};

macro_rules! impl_schema_custom_types {
    ($($t:ty),*) => {
        $(
            impl SchemaCustomTypes for $t {
            }
        )*
    };
}

macro_rules! impl_schema_errors {
    ($($t:ty),*) => {
        $(
            impl SchemaErrors for $t {
            }
        )*
    };
}

macro_rules! impl_schema_events {
    ($($t:ty),*) => {
        $(
            impl SchemaEvents for $t {
            }
        )*
    };
}

impl_schema_custom_types!(
    u8,
    u32,
    u64,
    i32,
    i64,
    U128,
    U256,
    U512,
    Address,
    bool,
    String,
    (),
    PublicKey,
    Bytes,
    URef
);

impl<T: CLTyped> SchemaCustomTypes for Option<T> {}

impl<T: CLTyped> SchemaCustomTypes for Vec<T> {}

impl<T: CLTyped, E: CLTyped> SchemaCustomTypes for Result<T, E> {}

impl<T: CLTyped, E: CLTyped> SchemaCustomTypes for BTreeMap<T, E> {}

impl<T1: CLTyped> SchemaCustomTypes for (T1,) {}

impl<T1: CLTyped, T2: CLTyped> SchemaCustomTypes for (T1, T2) {}

impl<T1: CLTyped, T2: CLTyped, T3: CLTyped> SchemaCustomTypes for (T1, T2, T3) {}

impl<const COUNT: usize> SchemaCustomTypes for [u8; COUNT] {}

impl<T: SchemaCustomTypes> SchemaCustomTypes for Maybe<T> {
    fn schema_types() -> Vec<Option<CustomType>> {
        T::schema_types()
    }
}

impl<M: SchemaErrors> SchemaErrors for SubModule<M> {
    fn schema_errors() -> Vec<casper_contract_schema::UserError> {
        M::schema_errors()
    }
}

impl_schema_errors!(
    u8,
    u32,
    u64,
    i32,
    i64,
    U128,
    U256,
    U512,
    Address,
    bool,
    String,
    (),
    PublicKey,
    Bytes
);

impl<T: CLTyped> SchemaErrors for Option<T> {}

impl<T: CLTyped> SchemaErrors for Vec<T> {}

impl<T: CLTyped, E: CLTyped> SchemaErrors for Result<T, E> {}

impl<T: CLTyped, E: CLTyped> SchemaErrors for BTreeMap<T, E> {}

impl<T1: CLTyped> SchemaErrors for (T1,) {}

impl<T1: CLTyped, T2: CLTyped> SchemaErrors for (T1, T2) {}

impl<T1: CLTyped, T2: CLTyped, T3: CLTyped> SchemaErrors for (T1, T2, T3) {}

impl<const COUNT: usize> SchemaErrors for [u8; COUNT] {}

impl<T: SchemaErrors> SchemaErrors for Maybe<T> {}

impl_schema_events!(
    u8,
    u32,
    u64,
    i32,
    i64,
    U128,
    U256,
    U512,
    Address,
    bool,
    String,
    (),
    PublicKey,
    Bytes
);

impl<T: CLTyped> SchemaEvents for Option<T> {}

impl<T: CLTyped> SchemaEvents for Vec<T> {}

impl<T: CLTyped, E: CLTyped> SchemaEvents for Result<T, E> {}

impl<T: CLTyped, E: CLTyped> SchemaEvents for BTreeMap<T, E> {}

impl<T1: CLTyped> SchemaEvents for (T1,) {}

impl<T1: CLTyped, T2: CLTyped> SchemaEvents for (T1, T2) {}

impl<T1: CLTyped, T2: CLTyped, T3: CLTyped> SchemaEvents for (T1, T2, T3) {}

impl<const COUNT: usize> SchemaEvents for [u8; COUNT] {}

impl<T: SchemaEvents> SchemaEvents for Maybe<T> {}

// I don't like it, but it's the only way to make it work
// If it was implemented it `core`, SchemaEvents would be implemented for ModulePrimitive
impl<M: SchemaErrors> SchemaErrors for Var<M> {}
impl<K: SchemaErrors, V> SchemaErrors for Mapping<K, V> {}
impl<V: SchemaErrors> SchemaErrors for List<V> {}
impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> SchemaErrors
    for Sequence<T>
{
}

impl<M: SchemaEvents> SchemaEvents for SubModule<M> {
    fn schema_events() -> Vec<casper_contract_schema::Event> {
        M::schema_events()
    }

    fn custom_types() -> Vec<Option<CustomType>> {
        M::custom_types()
    }
}
// I don't like it, but it's the only way to make it work
// If it was implemented it `core`, SchemaEvents would be implemented for ModulePrimitive
impl<M: SchemaEvents> SchemaEvents for Var<M> {}
impl<K: SchemaEvents, V> SchemaEvents for Mapping<K, V> {}
impl<V: SchemaEvents> SchemaEvents for List<V> {}
impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> SchemaEvents
    for Sequence<T>
{
}
