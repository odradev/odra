use casper_types::bytesrepr::{Error, FromBytes};
use casper_types::{bytesrepr::ToBytes, CLType, CLTyped};

const ADDRESS_LENGTH: usize = 64;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Address {
    data: [u8; ADDRESS_LENGTH],
}

impl Address {
    pub fn new(bytes: &[u8]) -> Address {
        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(ADDRESS_LENGTH, 0);

        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Address {
            data: bytes,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl CLTyped for Address {
    fn cl_type() -> casper_types::CLType {
        CLType::Any
    }
}

impl ToBytes for Address {
    fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(self.data.to_vec())
    }

    fn serialized_length(&self) -> usize {
        self.data.len()
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        let (data, remainder) = bytes.split_at(ADDRESS_LENGTH);
        Ok((Address::new(data), remainder))
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(&self.data);
        f.debug_struct("Address").field("data", &name).finish()
    }
}
