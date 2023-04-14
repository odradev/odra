use std::collections::BTreeMap;

use casper_types::{bytesrepr::Bytes, U128, U256, U512};

use crate::{Address, CallArgs};

pub trait OdraItem {
    const IS_ITEM: bool = false;

    fn events() -> Vec<String> {
        vec![]
    }
}

macro_rules! impl_is_mod {
    ($($t:ty)*) => {
        $(
            impl OdraItem for $t {}
        )*
    };
}

macro_rules! impl_is_mod_for_array {
    ($($N:literal)+) => {
        $(
            impl OdraItem for [u8; $N] {}
        )+
    }
}

impl_is_mod_for_array! {
      0  1  2  3  4  5  6  7  8  9
     10 11 12 13 14 15 16 17 18 19
     20 21 22 23 24 25 26 27 28 29
     30 31 32
     64 128 256 512
}

impl_is_mod!(bool i32 i64 u8 u32 u64 U128 U256 U512 &str String Address Bytes CallArgs);
impl OdraItem for () {}
impl<T: OdraItem> OdraItem for Option<T> {}
impl<T: OdraItem> OdraItem for Vec<T> {}
impl<T: OdraItem, E: OdraItem> OdraItem for Result<T, E> {}
impl<K: OdraItem, V: OdraItem> OdraItem for BTreeMap<K, V> {}
impl<T1: OdraItem> OdraItem for (T1,) {}
impl<T1: OdraItem, T2: OdraItem> OdraItem for (T1, T2) {}
impl<T1: OdraItem, T2: OdraItem, T3: OdraItem> OdraItem for (T1, T2, T3) {}
