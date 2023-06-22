use std::hash::Hasher;

use borsh::{BorshDeserialize, BorshSerialize};
use odra_types::address::OdraAddress;
use odra_types::AddressError;
use twox_hash::XxHash64;

use crate::key::Key;

/// Max bytes of an [`Address`] internal representation.
pub const ADDRESS_LENGTH: usize = 8;

/// Prefix for contract addresses.
pub const CONTRACT_ADDRESS_PREFIX: u32 = 0x0000cadd;

/// Blockchain-agnostic address representation.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSerialize, BorshDeserialize)]
pub struct Address {
    data: [u8; ADDRESS_LENGTH]
}

impl OdraAddress for Address {
    fn is_contract(&self) -> bool {
        // get first 4 bytes of data
        let bytes = self.data[0..4].try_into().unwrap();
        u32::from_be_bytes(bytes) == CONTRACT_ADDRESS_PREFIX
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for Address {
    type Error = AddressError;

    /// Creates a new Address from bytes.
    ///
    /// If passed less bytes than the capacity, the remaining bytes are zeroed.
    /// If passed more bytes and the capacity, the redundant bytes are discarded.
    fn try_from(bytes: &[u8; N]) -> Result<Self, Self::Error> {
        if bytes.is_empty() || bytes.iter().all(|&b| b == 0) {
            return Err(AddressError::ZeroAddress);
        }

        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(ADDRESS_LENGTH, 0);

        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Ok(Address { data: bytes })
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(self.data);
        f.debug_struct("Address").field("data", &name).finish()
    }
}

impl Key for Address {
    fn hash(&self) -> [u8; 8] {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&self.data);
        let result = hasher.finish();
        result.to_le_bytes()
    }
}