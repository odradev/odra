pub mod arithmetic;
pub mod contract_def;
mod error;
pub mod event;

pub use error::{CollectionError, ExecutionError, OdraError, VmError};
// TODO: Remove if possible.
/// Serialized event struct representation
pub type EventData = Vec<u8>;

#[derive(Debug, Clone)]
pub enum Type {
    Address,
    Bool,
    I32,
    I64,
    U8,
    U32,
    U64,
    U128,
    U256,
    U512,
    Unit,
    String,
    Option(Box<Type>),
    Result { ok: Box<Type>, err: Box<Type> },
    Map { key: Box<Type>, value: Box<Type> },
    Tuple1([Box<Type>; 1]),
    Tuple2([Box<Type>; 2]),
    Tuple3([Box<Type>; 3]),
    Any,
    Vec(Box<Type>),
}
