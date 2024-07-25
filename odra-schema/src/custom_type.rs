use std::collections::BTreeMap;

use casper_contract_schema::CustomType;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::{bytesrepr::Bytes, CLTyped, PublicKey, U128, U256, U512};
use casper_types::{Key, URef};
use num_traits::{Num, One, Zero};
use odra_core::{args::Maybe, Address};
use odra_core::{ContractRef, External, List, Mapping, Sequence, SubModule, Var};

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

impl<T: SchemaCustomTypes> SchemaCustomTypes for Option<T> {
    fn schema_types() -> Vec<Option<CustomType>> {
        T::schema_types()
    }
}

impl<T: SchemaCustomTypes> SchemaCustomTypes for Vec<T> {
    fn schema_types() -> Vec<Option<CustomType>> {
        T::schema_types()
    }
}

impl<T: SchemaCustomTypes, E: SchemaCustomTypes> SchemaCustomTypes for Result<T, E> {
    fn schema_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T::schema_types());
        types.extend(E::schema_types());
        types
    }
}

impl<T: SchemaCustomTypes, E: SchemaCustomTypes> SchemaCustomTypes for BTreeMap<T, E> {
    fn schema_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T::schema_types());
        types.extend(E::schema_types());
        types
    }
}

impl<T1: SchemaCustomTypes> SchemaCustomTypes for (T1,) {
    fn schema_types() -> Vec<Option<CustomType>> {
        T1::schema_types()
    }
}

impl<T1: SchemaCustomTypes, T2: SchemaCustomTypes> SchemaCustomTypes for (T1, T2) {
    fn schema_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T1::schema_types());
        types.extend(T2::schema_types());
        types
    }
}

impl<T1: SchemaCustomTypes, T2: SchemaCustomTypes, T3: SchemaCustomTypes> SchemaCustomTypes
    for (T1, T2, T3)
{
    fn schema_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T1::schema_types());
        types.extend(T2::schema_types());
        types.extend(T3::schema_types());
        types
    }
}

impl SchemaCustomTypes for Key {}

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

impl<T: SchemaCustomTypes> SchemaEvents for Option<T> {
    fn custom_types() -> Vec<Option<CustomType>> {
        T::schema_types()
    }
}

impl<T: SchemaCustomTypes> SchemaEvents for Vec<T> {
    fn custom_types() -> Vec<Option<CustomType>> {
        T::schema_types()
    }
}

impl<T: SchemaCustomTypes, E: SchemaCustomTypes> SchemaEvents for Result<T, E> {
    fn custom_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T::schema_types());
        types.extend(E::schema_types());
        types
    }
}

impl<T: SchemaCustomTypes, E: SchemaCustomTypes> SchemaEvents for BTreeMap<T, E> {
    fn custom_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T::schema_types());
        types.extend(E::schema_types());
        types
    }
}

impl<T1: SchemaCustomTypes> SchemaEvents for (T1,) {
    fn custom_types() -> Vec<Option<CustomType>> {
        T1::schema_types()
    }
}

impl<T1: SchemaCustomTypes, T2: SchemaCustomTypes> SchemaEvents for (T1, T2) {
    fn custom_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T1::schema_types());
        types.extend(T2::schema_types());
        types
    }
}

impl<T1: SchemaCustomTypes, T2: SchemaCustomTypes, T3: SchemaCustomTypes> SchemaEvents
    for (T1, T2, T3)
{
    fn custom_types() -> Vec<Option<CustomType>> {
        let mut types = vec![];
        types.extend(T1::schema_types());
        types.extend(T2::schema_types());
        types.extend(T3::schema_types());
        types
    }
}

impl<const COUNT: usize> SchemaEvents for [u8; COUNT] {}

impl<T: SchemaEvents> SchemaEvents for Maybe<T> {}

// I don't like it, but it's the only way to make it work
// If it was implemented it `core`, SchemaEvents would be implemented for ModulePrimitive
impl<M: SchemaErrors + ContractRef> SchemaErrors for External<M> {}
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
impl<M: SchemaEvents + ContractRef> SchemaEvents for External<M> {}
impl<M: SchemaEvents> SchemaEvents for Var<M> {}
impl<K: SchemaEvents, V> SchemaEvents for Mapping<K, V> {}
impl<V: SchemaEvents> SchemaEvents for List<V> {}
impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> SchemaEvents
    for Sequence<T>
{
}
