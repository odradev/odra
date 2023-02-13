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

pub trait AsString {
    fn as_string(&self) -> String;
}

macro_rules! impl_as_string {
    ( $( $name:ident ),+ ) => {
        $(impl AsString for $name {
            fn as_string(&self) -> String {
                self.to_string()
            }
        })+
    };
}

impl_as_string!(u8, u32, i32, u64, U256, U512, bool, String, Address);

impl<T: AsString> AsString for Option<T> {
    fn as_string(&self) -> String {
        match self {
            Some(value) => value.as_string(),
            None => String::from("null")
        }
    }
}

impl AsString for () {
    fn as_string(&self) -> String {
        String::from("")
    }
}

impl AsString for &str {
    fn as_string(&self) -> String {
        String::from(*self)
    }
}

impl<T: AsString> AsString for Vec<T> {
    fn as_string(&self) -> String {
        let values = self
            .iter()
            .map(|value| value.as_string())
            .collect::<Vec<_>>();
        let values = values.join(",");
        format!("[{}]", values)
    }
}
