use std::collections::BTreeMap;

use odra_types::contract_def::Event;

use crate::types::{Address, U256, U512};

pub trait OdraItem {
    fn is_module() -> bool;

    fn events() -> Vec<Event>;
}

macro_rules! impl_odra_item_for_types {
    ($($t:ty),+) => {
        $(
            impl OdraItem for $t {
                fn is_module() -> bool {
                    false
                }

                fn events() -> Vec<Event> {
                    vec![]
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

                fn events() -> Vec<Event> {
                    vec![]
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

impl_odra_item_for_types!(Address, String, bool, i32, i64, u8, u32, u64, u128, U256, U512, ());

impl<T> OdraItem for Option<T> {
    fn is_module() -> bool {
        false
    }

    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T, E> OdraItem for Result<T, E> {
    fn is_module() -> bool {
        false
    }

    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T, E> OdraItem for BTreeMap<T, E> {
    fn is_module() -> bool {
        false
    }

    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T> OdraItem for Vec<T> {
    fn is_module() -> bool {
        false
    }

    fn events() -> Vec<Event> {
        vec![]
    }
}
