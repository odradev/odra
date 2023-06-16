extern crate core;

mod address;
mod call_args;
mod mock_vm_type;
mod ty;
#[allow(clippy::assign_op_pattern, clippy::reversed_empty_ranges)]
mod uints;

pub use address::Address;
pub use address::CONTRACT_ADDRESS_PREFIX;
pub use borsh::{
    maybestd::io::{Error, ErrorKind, Result, Write},
    BorshDeserialize, BorshSerialize
};
pub use call_args::CallArgs;
pub use mock_vm_type::{MockDeserializable, MockSerializable, MockVMSerializationError};
use std::ops::Deref;
pub use ty::Typed;
pub use uints::{U128, U256, U512};
/// A type representing the amount of native tokens.
pub type Balance = U256;
/// A type representing the block time.
pub type BlockTime = u64;

/// A type representing the public key. Caution: MockVM does not implement any cryptography!
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PublicKey(pub Address);

impl PublicKey {
    pub fn inner_bytes(&self) -> &[u8] {
        self.0.inner_bytes()
    }
}

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

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
