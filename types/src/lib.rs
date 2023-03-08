//! Blockchain-agnostic types used by Odra Framework.

pub mod address;
pub mod arithmetic;
pub mod contract_def;
mod error;
pub mod event;

pub use error::{AddressError, CollectionError, ExecutionError, OdraError, VmError};

/// Types accepted by Odra framework, these types can be stored and manipulated by smart contracts.
#[derive(Debug, Clone)]
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
    Vec(Box<Type>)
}
