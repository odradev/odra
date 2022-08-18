#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod address;
pub mod arithmetic;
mod error;
pub mod event;
mod serde;

pub use address::*;
pub use casper_types::{
    runtime_args, ApiError, CLType, CLTyped, CLValue, CLValueError, NamedArg, RuntimeArgs, U128,
    U256, U512,
};
pub use error::{ExecutionError, OdraError, VmError};
pub use serde::{FromBytes, ToBytes};
/// Contains serialization and deserialization code for types used throughout the system.
pub mod bytesrepr {
    pub use casper_types::bytesrepr::{deserialize, serialize, Bytes, Error, FromBytes, ToBytes};
}
/// Serialized event struct representation
#[cfg(not(feature = "std"))]
pub type EventData = alloc::vec::Vec<u8>;
/// Serialized event struct representation
#[cfg(feature = "std")]
pub type EventData = std::vec::Vec<u8>;
