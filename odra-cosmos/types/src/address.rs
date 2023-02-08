//! Better address representation for Casper.

use cosmwasm_std::Addr;
use serde::{
    de::{value::BytesDeserializer, Visitor},
    Deserialize, Serialize
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Address([u8; 20]);

impl Address {
    pub fn new(bytes: &[u8]) -> Address {
        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(20, 0);

        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Address(bytes)
    }
}

impl Into<Addr> for Address {
    fn into(self) -> Addr {
        Addr::unchecked(self.to_string())
    }
}

impl Into<String> for Address {
    fn into(self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }
}

impl Into<String> for &Address {
    fn into(self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(self.to_string());
        f.debug_struct("Address").field("data", &name).finish()
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let str: String = self.into();
        serializer.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let bytes = deserializer.deserialize_str(AddressVisitor)?;
        Ok(Address::new(bytes.as_bytes()))
    }
}

struct AddressVisitor;

impl<'de> Visitor<'de> for AddressVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a &str")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error
    {
        Ok(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Address, CosmosType, OdraType};

    #[test]
    fn serde() {
        let address = Address::new(b"some address");

        let json = serde_json::to_vec(&address).unwrap();
        assert_eq!(address, serde_json::from_slice(&json).unwrap());
    }
}
