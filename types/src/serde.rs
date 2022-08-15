#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// A type which can be serialized to a `Vec<u8>`.
pub trait ToBytes: Sized {
    /// Serialization failure reason.
    type Error;
    /// Serializes self to a `Vec<u8>`.
    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
}

/// A type which can be deserialized from a `Vec<u8>`.
pub trait FromBytes: Sized {
    /// Deserialization failure reason.
    type Error;
    /// A type the bytes are serialized into.
    type Item;
    /// Deserializes a `Vec<u8>` into `Self::Item`.
    fn deserialize(data: Vec<u8>) -> Result<(Self::Item, Vec<u8>), Self::Error>;
}
