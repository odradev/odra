mod address;
mod bytes;
mod call_args;
mod cosmos_type;
mod ty;
#[allow(clippy::assign_op_pattern)]
mod uints;

pub use address::Address;
pub use bytes::Bytes;
pub use call_args::CallArgs;
pub use cosmos_type::CosmosSerializationError;
pub use cosmos_type::CosmosType;
use odra_types::event::OdraEvent;
pub use serde::{Deserialize, Serialize};
pub use ty::{AsString, Typed};
pub use uints::{U256, U512};
/// A type representing the amount of native tokens.
pub type Balance = U256;
/// A type representing the block time.
pub type BlockTime = u64;

/// A type that can be written to the storage and read from the storage.
pub trait OdraType: CosmosType + AsString {}

impl<T: CosmosType + AsString> OdraType for T {}

pub trait SerializableEvent: OdraEvent + From<cosmwasm_std::Event> {}

/// Represents a serialized event.
pub type EventData = Vec<u8>;
