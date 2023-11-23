use crate::prelude::{collections::BTreeMap, string::String, vec::Vec};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    U128, U256, U512
};

use crate::Address;

pub trait OdraItem {
    fn is_module() -> bool;

    #[cfg(not(target_arch = "wasm32"))]
    fn events() -> Vec<crate::contract_def::Event> {
        crate::prelude::vec![]
    }
}

macro_rules! impl_odra_item_for_types {
    ($($t:ty),+) => {
        $(
            impl OdraItem for $t {
                fn is_module() -> bool {
                    false
                }
            }
        )+
    };
}

macro_rules! impl_odra_item_for_array_types {
    ($($n:expr)+) => {
        $(
            impl<T> OdraItem for [T; $n] {
                fn is_module() -> bool {
                    false
                }
            }
        )+
    };
}

impl_odra_item_for_array_types!(
    0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
    64 128 256 512
);

impl_odra_item_for_types!(
    Address,
    String,
    bool,
    i32,
    i64,
    u8,
    u32,
    u64,
    U128,
    U256,
    U512,
    ()
);

impl<T: ToBytes + FromBytes> OdraItem for Option<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T: ToBytes + FromBytes, E: ToBytes + FromBytes> OdraItem for Result<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T: ToBytes + FromBytes, E: ToBytes + FromBytes> OdraItem for BTreeMap<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T: ToBytes + FromBytes> OdraItem for Vec<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T1: ToBytes + FromBytes> OdraItem for (T1,) {
    fn is_module() -> bool {
        false
    }
}

impl<T1: ToBytes + FromBytes, T2: ToBytes + FromBytes> OdraItem for (T1, T2) {
    fn is_module() -> bool {
        false
    }
}

impl<T1: ToBytes + FromBytes, T2: ToBytes + FromBytes, T3: ToBytes + FromBytes> OdraItem
    for (T1, T2, T3)
{
    fn is_module() -> bool {
        false
    }
}
