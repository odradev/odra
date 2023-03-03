use borsh::{BorshDeserialize, BorshSerialize};
use odra_types::AddressError;

/// Max bytes of an [`Address`] internal representation.
pub const ADDRESS_LENGTH: usize = 8;

/// Blockchain-agnostic address representation.
#[derive(Clone, Copy, PartialEq, Hash, Eq, BorshSerialize, BorshDeserialize)]
pub struct Address {
    data: [u8; ADDRESS_LENGTH]
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
