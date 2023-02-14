mod address;
mod call_args;
mod mock_vm_type;
mod ty;
#[allow(clippy::assign_op_pattern)]
mod uints;

pub use address::Address;
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use call_args::CallArgs;
pub use mock_vm_type::{MockVMSerializationError, MockVMType};
use odra_types::event::OdraEvent;
pub use ty::Typed;
pub use uints::{U256, U512};
/// A type representing the amount of native tokens.
pub type Balance = U256;
/// A type representing the block time.
pub type BlockTime = u64;

/// A type that can be written to the storage and read from the storage.
pub trait OdraType: MockVMType {}

impl<T: MockVMType> OdraType for T {}

pub trait SerializableEvent: OdraEvent + OdraType {}

/// Represents a serialized event.
pub type EventData = Vec<u8>;
