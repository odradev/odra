use casper_types::bytesrepr::{Error, FromBytes};
use casper_types::{bytesrepr::ToBytes, CLType, CLTyped};

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Address {
    pub data: Vec<u8>,
}

impl Address {
    pub fn new(bytes: &[u8]) -> Address {
        Address {
            data: bytes.to_vec(),
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
        Ok(self.data.clone())
    }

    fn serialized_length(&self) -> usize {
        self.data.len()
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), Error> {
        Ok((Address::new(bytes), &[]))
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(&self.data);
        f.debug_struct("Address").field("data", &name).finish()
    }
}
