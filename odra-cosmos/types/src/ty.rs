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

// impl<T: Typed> ToValue for T {
//     fn to_value(&self) -> serde_json::Value {
//         match <T as Typed>::ty() {
//             Type::Address => serde_json::Value::String(&self.to_string()),
//             Type::Bool => todo!(),
//             Type::I32 => todo!(),
//             Type::I64 => todo!(),
//             Type::U8 => todo!(),
//             Type::U32 => todo!(),
//             Type::U64 => todo!(),
//             Type::U128 => todo!(),
//             Type::U256 => todo!(),
//             Type::U512 => todo!(),
//             Type::Unit => todo!(),
//             Type::String => todo!(),
//             Type::Option(_) => todo!(),
//             Type::Result { ok, err } => todo!(),
//             Type::Map { key, value } => todo!(),
//             Type::Tuple1(_) => todo!(),
//             Type::Tuple2(_) => todo!(),
//             Type::Tuple3(_) => todo!(),
//             Type::Any => todo!(),
//             Type::Vec(_) => todo!(),
//         }
//     }
// }
