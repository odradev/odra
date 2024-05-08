use crate::{
    prelude::{collections::BTreeMap, string::String, vec::Vec},
    types::{Address, OdraType, U256, U512}
};

pub trait OdraItem {
    fn is_module() -> bool;

    #[cfg(not(target_arch = "wasm32"))]
    fn events() -> Vec<odra_types::contract_def::Event> {
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
    u128,
    U256,
    U512,
    ()
);

impl<T: OdraType> OdraItem for Option<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T: OdraType, E: OdraType> OdraItem for Result<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T: OdraType, E: OdraType> OdraItem for BTreeMap<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T: OdraType> OdraItem for Vec<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T1: OdraType> OdraItem for (T1,) {
    fn is_module() -> bool {
        false
    }
}

impl<T1: OdraType, T2: OdraType> OdraItem for (T1, T2) {
    fn is_module() -> bool {
        false
    }
}

impl<T1: OdraType, T2: OdraType, T3: OdraType> OdraItem for (T1, T2, T3) {
    fn is_module() -> bool {
        false
    }
}
