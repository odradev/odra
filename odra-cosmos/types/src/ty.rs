use odra_types::Type;

use crate::{Address, U256, U512};

/// A type which can be described as a [Type].
pub trait Typed {
    fn ty() -> Type;
}

impl Typed for u8 {
    fn ty() -> Type {
        Type::U8
    }
}

impl Typed for u32 {
    fn ty() -> Type {
        Type::U32
    }
}

impl Typed for i32 {
    fn ty() -> Type {
        Type::I32
    }
}

impl Typed for u64 {
    fn ty() -> Type {
        Type::U64
    }
}

impl Typed for U256 {
    fn ty() -> Type {
        Type::U256
    }
}

impl Typed for U512 {
    fn ty() -> Type {
        Type::U512
    }
}

impl Typed for bool {
    fn ty() -> Type {
        Type::Bool
    }
}

impl Typed for () {
    fn ty() -> Type {
        Type::Unit
    }
}

impl Typed for String {
    fn ty() -> Type {
        Type::String
    }
}

impl Typed for Address {
    fn ty() -> Type {
        Type::Address
    }
}

impl<T> Typed for Option<T>
where
    T: Typed
{
    fn ty() -> Type {
        Type::Option(Box::new(T::ty()))
    }
}

impl<T> Typed for Vec<T>
where
    T: Typed
{
    fn ty() -> Type {
        Type::Vec(Box::new(T::ty()))
    }
}

trait ToValue {
    fn to_value(&self) -> serde_json::Value;
}
impl ToValue for Address {
    fn to_value(&self) -> serde_json::Value {
        let str: String = self.into();
        serde_json::Value::String(str)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> serde_json::Value {
        serde_json::Value::Bool(*self)
    }
}
