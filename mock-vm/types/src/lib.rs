extern crate core;

mod address;
mod call_args;
mod mock_vm_type;
mod ty;
#[allow(clippy::assign_op_pattern, clippy::reversed_empty_ranges)]
mod uints;

pub use address::Address;
pub use address::CONTRACT_ADDRESS_PREFIX;
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use call_args::CallArgs;
pub use mock_vm_type::{MockDeserializable, MockSerializable, MockVMSerializationError};
pub use ty::Typed;
pub use uints::{U128, U256, U512};
/// A type representing the amount of native tokens.
pub type Balance = U256;
/// A type representing the block time.
pub type BlockTime = u64;

/// A type representing the public key. Caution: MockVM does not implement any cryptography!
pub struct PublicKey(pub [u8; 8]);

/// A type that can be written to the storage and read from the storage.
pub trait OdraType: MockSerializable + MockDeserializable {
    /// Serializes the value.
    fn serialize(&self) -> Option<Vec<u8>> {
        self.ser().ok()
    }

    /// Deserializes the value.
    fn deserialize(bytes: &[u8]) -> Option<Self> {
        Self::deser(bytes.to_vec()).ok()
    }
}

impl<T: MockSerializable + MockDeserializable> OdraType for T {}

/// Represents a serialized event.
pub type EventData = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize, Default)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    /// Constructs a new, empty vector of bytes.
    pub fn new() -> Bytes {
        Bytes::default()
    }

    /// Returns reference to inner container.
    #[inline]
    pub fn inner_bytes(&self) -> &Vec<u8> {
        &self.0
    }

    /// Extracts a slice containing the entire vector.
    pub fn as_slice(&self) -> &[u8] {
        self.inner_bytes().as_slice()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}
