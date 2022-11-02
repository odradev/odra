use odra_types::Type;

use crate::{Address, U256, U512};

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
