use odra_types::Type;

use crate::{Address, Bytes, U256, U512, uints::U128};

/// A trait that adds [Type] description for a given type.
pub trait Typed {
    fn ty() -> Type;
}

macro_rules! impl_typed {
    ($($ty:ty, $tt:expr),*) => {
        $(
            impl Typed for $ty {
                fn ty() -> Type {
                    $tt
                }
            }
        )*
    };
}

impl_typed!(
    i32, Type::I32,
    i64, Type::I64,
    u8, Type::U8,
    u32, Type::U32,
    u64, Type::U64,
    U128, Type::U128,
    U256, Type::U256,
    U512, Type::U512,
    bool, Type::Bool,
    (), Type::Unit,
    String, Type::String,
    Address, Type::Address,
    Bytes, Type::Vec(Box::new(Type::U8))
);

macro_rules! impl_typed_for_array {
    ($($n:expr)*) => {
        $(
            impl<T: Typed> Typed for [T; $n] {
                fn ty() -> Type {
                    Type::Vec(Box::new(Type::U8))
                }
            }
        )*
    };
}

impl_typed_for_array!(
    0  1  2  3  4  5  6  7  8  9
     10 11 12 13 14 15 16 17 18 19
     20 21 22 23 24 25 26 27 28 29
     30 31 32
     64 128 256 512
);

impl<T: Typed> Typed for Option<T> {
    fn ty() -> Type {
        Type::Option(Box::new(T::ty()))
    }
}

impl<T: Typed> Typed for Vec<T> {
    fn ty() -> Type {
        Type::Vec(Box::new(T::ty()))
    }
}

impl<T: Typed, E: Typed> Typed for Result<T, E> {
    fn ty() -> Type {
        Type::Result {
            ok: Box::new(T::ty()),
            err: Box::new(E::ty())
        }
    }
}

impl<K: Typed, V: Typed> Typed for std::collections::BTreeMap<K, V> {
    fn ty() -> Type {
        Type::Map {
            key: Box::new(K::ty()),
            value: Box::new(V::ty())
        }
    }
}

impl<T: Typed> Typed for (T,) {
    fn ty() -> Type {
        Type::Tuple1([Box::new(T::ty())])
    }
}

impl<T: Typed, U: Typed> Typed for (T, U) {
    fn ty() -> Type {
        Type::Tuple2([Box::new(T::ty()), Box::new(U::ty())])
    }
}

impl<T: Typed, U: Typed, V: Typed> Typed for (T, U, V) {
    fn ty() -> Type {
        Type::Tuple3([Box::new(T::ty()), Box::new(U::ty()), Box::new(V::ty())])
    }
}
