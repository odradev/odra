use std::ops::Deref;

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

#[derive(Default)]
pub struct Bytes(casper_types::bytesrepr::Bytes);

impl CLTyped for Bytes {
    fn cl_type() -> casper_types::CLType {
        <Vec<u8>>::cl_type()
    }
}

impl ToBytes for Bytes {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for Bytes {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        casper_types::bytesrepr::Bytes::from_bytes(bytes)
            .map(|(bytes, leftovers)| (Bytes(bytes), leftovers))
    }
}

impl Deref for Bytes {
    type Target = casper_types::bytesrepr::Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
