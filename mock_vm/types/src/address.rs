use casper_types::{bytesrepr::{FromBytes, ToBytes}, CLType, CLTyped};

use crate::OdraType;

/// Max bytes of an [`Address`] internal representation.
const ADDRESS_LENGTH: usize = 4;

/// Blockchain-agnostic address representation.
#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Address {
    data: [u8; ADDRESS_LENGTH],
}

impl Address {
    /// Creates a new Address from bytes.
    ///
    /// If takes less than [`ADDRESS_LENGTH`], the remaining bytes are zeroed.
    /// If takes more and [`ADDRESS_LENGTH`] excess bytes are discarded.
    pub fn new(bytes: &[u8]) -> Address {
        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(ADDRESS_LENGTH, 0);

        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Address { data: bytes }
    }

    /// Returns a slice containing the entire array of bytes.
    pub fn bytes(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(&self.data);
        f.debug_struct("Address").field("data", &name).finish()
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        Ok((Address {
            data: bytes.try_into().expect("cant convert")
        }, &[]))
    }
}

impl ToBytes for Address {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        Ok(self.bytes().to_vec())
    }

    fn serialized_length(&self) -> usize {
        self.bytes().len()
    }
}

impl CLTyped for Address {
    fn cl_type() -> CLType {
        CLType::Any
    }
}

impl OdraType for Address {}
