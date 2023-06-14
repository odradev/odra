//! Blockchain-agnostic types used by Odra Framework.

#![no_std]

extern crate alloc;

pub mod address;
pub mod arithmetic;
pub mod contract_def;
mod error;
pub mod event;

use alloc::boxed::Box;
pub use error::{AddressError, CollectionError, ExecutionError, OdraError, VmError};

/// Types accepted by Odra framework, these types can be stored and manipulated by smart contracts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    /// Address type.
    Address,
    /// `bool` primitive.
    Bool,
    /// `i32` primitive.
    I32,
    /// `i64` primitive.
    I64,
    /// `u8` primitive.
    U8,
    /// `u32` primitive.
    U32,
    /// `u64` primitive.
    U64,
    /// `U128` large unsigned integer type.
    U128,
    /// `U256` large unsigned integer type.
    U256,
    /// `U512` large unsigned integer type.
    U512,
    /// `()` primitive.
    Unit,
    /// `String` primitive.
    String,
    /// `Option` of a `Type`.
    Option(Box<Type>),
    /// `Result` with `Ok` and `Err` variants of `Type`s.
    Result { ok: Box<Type>, err: Box<Type> },
    /// Map with keys of a `Type` and values of a `Type`.
    Map { key: Box<Type>, value: Box<Type> },
    /// 1-ary tuple of a `Type`.
    Tuple1([Box<Type>; 1]),
    /// 2-ary tuple of a `Type`.
    Tuple2([Box<Type>; 2]),
    /// 3-ary tuple of a `Type`.
    Tuple3([Box<Type>; 3]),
    /// Unspecified type.
    Any,
    /// Vector of a `Type`.
    Vec(Box<Type>),
    /// Fixed-length list of a single `Type`.
    ByteArray(u32),
    /// A slice of a `Type`.
    Slice(Box<Type>)
}

impl Type {
    fn has_any(ty: &Type) -> bool {
        match ty {
            // Positive if depth is not, which means this is the main structure of the event.
            Type::Any => true,

            // Negative.
            Type::Bool
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::U256
            | Type::U512
            | Type::Unit
            | Type::String
            | Type::ByteArray(_) 
            | Type::Address => false,

            // Need recursive check.
            Type::Option(ty) => Type::has_any(ty),
            Type::Vec(ty) => Type::has_any(ty),
            Type::Slice(ty) => Type::has_any(ty),
            Type::Result { ok, err } => Type::has_any(ok) || Type::has_any(err),
            Type::Map { key, value } => Type::has_any(key) || Type::has_any(value),
            Type::Tuple1([ty]) => Type::has_any(ty),
            Type::Tuple2([ty1, ty2]) => Type::has_any(ty1) || Type::has_any(ty2),
            Type::Tuple3([ty1, ty2, ty3]) => Type::has_any(ty1) || Type::has_any(ty2) || Type::has_any(ty3),
        }
    }
}
