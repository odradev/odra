use std::collections::BTreeMap;

use crate::types::{Address, U256, U512};

pub trait IsModule {
    fn is_module() -> bool;
}

// Macro that implementes IsModule for a single type with given bool value.
macro_rules! impl_is_module {
    ($type:ty, $value:expr) => {
        impl IsModule for $type {
            fn is_module() -> bool {
                $value
            }
        }
    };
}

impl_is_module!((), false);
impl_is_module!(bool, false);
impl_is_module!(i32, false);
impl_is_module!(i64, false);
impl_is_module!(u8, false);
impl_is_module!(u32, false);
impl_is_module!(u64, false);
impl_is_module!(u128, false);
impl_is_module!(U256, false);
impl_is_module!(U512, false);
impl_is_module!(String, false);
impl_is_module!(Address, false);

impl<T> IsModule for Option<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T, E> IsModule for Result<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T, E> IsModule for BTreeMap<T, E> {
    fn is_module() -> bool {
        false
    }
}

impl<T> IsModule for Vec<T> {
    fn is_module() -> bool {
        false
    }
}

impl<T> IsModule for [T; 1] {
    fn is_module() -> bool {
        false
    }
}

impl<T> IsModule for [T; 2] {
    fn is_module() -> bool {
        false
    }
}

impl<T> IsModule for [T; 3] {
    fn is_module() -> bool {
        false
    }
}
