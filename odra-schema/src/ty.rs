use std::collections::BTreeMap;

pub use casper_contract_schema;
use casper_contract_schema::NamedCLType;
use casper_types::{bytesrepr::Bytes, ContractHash, Key, PublicKey, URef, U128, U256, U512};
use odra_core::{args::Maybe, Address};

/// Trait for types that can be represented as a NamedCLType.
pub trait NamedCLTyped {
    /// Returns the NamedCLType of the implementing type.
    fn ty() -> NamedCLType;
}

impl NamedCLTyped for bool {
    fn ty() -> NamedCLType {
        NamedCLType::Bool
    }
}

impl NamedCLTyped for i32 {
    fn ty() -> NamedCLType {
        NamedCLType::I32
    }
}

impl NamedCLTyped for i64 {
    fn ty() -> NamedCLType {
        NamedCLType::I64
    }
}

impl NamedCLTyped for u8 {
    fn ty() -> NamedCLType {
        NamedCLType::U8
    }
}

impl NamedCLTyped for u32 {
    fn ty() -> NamedCLType {
        NamedCLType::U32
    }
}

impl NamedCLTyped for u64 {
    fn ty() -> NamedCLType {
        NamedCLType::U64
    }
}

impl NamedCLTyped for U128 {
    fn ty() -> NamedCLType {
        NamedCLType::U128
    }
}

impl NamedCLTyped for U256 {
    fn ty() -> NamedCLType {
        NamedCLType::U256
    }
}

impl NamedCLTyped for U512 {
    fn ty() -> NamedCLType {
        NamedCLType::U512
    }
}

impl NamedCLTyped for () {
    fn ty() -> NamedCLType {
        NamedCLType::Unit
    }
}

impl NamedCLTyped for String {
    fn ty() -> NamedCLType {
        NamedCLType::String
    }
}

impl NamedCLTyped for &str {
    fn ty() -> NamedCLType {
        NamedCLType::String
    }
}

impl NamedCLTyped for Key {
    fn ty() -> NamedCLType {
        NamedCLType::Key
    }
}

impl NamedCLTyped for URef {
    fn ty() -> NamedCLType {
        NamedCLType::URef
    }
}

impl NamedCLTyped for Bytes {
    fn ty() -> NamedCLType {
        <Vec<u8> as NamedCLTyped>::ty()
    }
}

impl<T: NamedCLTyped> NamedCLTyped for Option<T> {
    fn ty() -> NamedCLType {
        NamedCLType::Option(Box::new(T::ty()))
    }
}

impl<T: NamedCLTyped> NamedCLTyped for Vec<T> {
    fn ty() -> NamedCLType {
        NamedCLType::List(Box::new(T::ty()))
    }
}

impl<const COUNT: usize> NamedCLTyped for [u8; COUNT] {
    fn ty() -> NamedCLType {
        NamedCLType::ByteArray(COUNT as u32)
    }
}

impl<T: NamedCLTyped, E: NamedCLTyped> NamedCLTyped for Result<T, E> {
    fn ty() -> NamedCLType {
        let ok = Box::new(T::ty());
        let err = Box::new(E::ty());
        NamedCLType::Result { ok, err }
    }
}

impl<K: NamedCLTyped, V: NamedCLTyped> NamedCLTyped for BTreeMap<K, V> {
    fn ty() -> NamedCLType {
        let key = Box::new(K::ty());
        let value = Box::new(V::ty());
        NamedCLType::Map { key, value }
    }
}

impl<T1: NamedCLTyped> NamedCLTyped for (T1,) {
    fn ty() -> NamedCLType {
        NamedCLType::Tuple1([Box::new(T1::ty())])
    }
}

impl<T1: NamedCLTyped, T2: NamedCLTyped> NamedCLTyped for (T1, T2) {
    fn ty() -> NamedCLType {
        NamedCLType::Tuple2([Box::new(T1::ty()), Box::new(T2::ty())])
    }
}

impl<T1: NamedCLTyped, T2: NamedCLTyped, T3: NamedCLTyped> NamedCLTyped for (T1, T2, T3) {
    fn ty() -> NamedCLType {
        NamedCLType::Tuple3([Box::new(T1::ty()), Box::new(T2::ty()), Box::new(T3::ty())])
    }
}

impl NamedCLTyped for PublicKey {
    fn ty() -> NamedCLType {
        NamedCLType::PublicKey
    }
}

impl NamedCLTyped for ContractHash {
    fn ty() -> NamedCLType {
        NamedCLType::ByteArray(32)
    }
}

impl NamedCLTyped for Address {
    fn ty() -> NamedCLType {
        NamedCLType::Key
    }
}

impl<T: NamedCLTyped> NamedCLTyped for Maybe<T> {
    fn ty() -> NamedCLType {
        T::ty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_named_cl_type() {
        assert_eq!(bool::ty(), NamedCLType::Bool);
        assert_eq!(i32::ty(), NamedCLType::I32);
        assert_eq!(i64::ty(), NamedCLType::I64);
        assert_eq!(u8::ty(), NamedCLType::U8);
        assert_eq!(u32::ty(), NamedCLType::U32);
        assert_eq!(u64::ty(), NamedCLType::U64);
        assert_eq!(U128::ty(), NamedCLType::U128);
        assert_eq!(U256::ty(), NamedCLType::U256);
        assert_eq!(U512::ty(), NamedCLType::U512);
        assert_eq!(<() as NamedCLTyped>::ty(), NamedCLType::Unit);
        assert_eq!(String::ty(), NamedCLType::String);
        assert_eq!(Key::ty(), NamedCLType::Key);
        assert_eq!(URef::ty(), NamedCLType::URef);
        assert_eq!(PublicKey::ty(), NamedCLType::PublicKey);
        assert_eq!(<&str as NamedCLTyped>::ty(), NamedCLType::String);
        assert_eq!(Bytes::ty(), NamedCLType::List(Box::new(NamedCLType::U8)));
        assert_eq!(
            Option::<u32>::ty(),
            NamedCLType::Option(Box::new(NamedCLType::U32))
        );
        assert_eq!(
            Vec::<u32>::ty(),
            NamedCLType::List(Box::new(NamedCLType::U32))
        );
        assert_eq!(<[u8; 32] as NamedCLTyped>::ty(), NamedCLType::ByteArray(32));
        assert_eq!(
            Result::<u32, u64>::ty(),
            NamedCLType::Result {
                ok: Box::new(NamedCLType::U32),
                err: Box::new(NamedCLType::U64)
            }
        );
        assert_eq!(
            BTreeMap::<u32, u64>::ty(),
            NamedCLType::Map {
                key: Box::new(NamedCLType::U32),
                value: Box::new(NamedCLType::U64)
            }
        );
        assert_eq!(
            <(u32,) as NamedCLTyped>::ty(),
            NamedCLType::Tuple1([Box::new(NamedCLType::U32)])
        );
        assert_eq!(
            <(u32, u64) as NamedCLTyped>::ty(),
            NamedCLType::Tuple2([Box::new(NamedCLType::U32), Box::new(NamedCLType::U64)])
        );
        assert_eq!(
            <(u32, u64, u8) as NamedCLTyped>::ty(),
            NamedCLType::Tuple3([
                Box::new(NamedCLType::U32),
                Box::new(NamedCLType::U64),
                Box::new(NamedCLType::U8)
            ])
        );
    }
}
