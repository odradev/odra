//! Address representation for Cosmos.

use std::str::FromStr;

use cosmwasm_std::Addr;
use serde::{de::Visitor, Deserialize, Serialize};

const ADDRESS_LENGTH: usize = 90;

/// Internal address representation.
///
/// Typically an `Address` should be created from cosmwasm [Addr].
///
/// In Cosmos, an address is typically bech32 encoded. For other cosmos-based chains
/// smart contracts no assumptions should be made other than being UTF-8 encoded
/// and of reasonable length.
///
/// Read more in [`comsmwasm_std docs`](Addr).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Address([u8; ADDRESS_LENGTH], u8);

impl Address {
    pub fn new(bytes: &[u8]) -> Address {
        let len = bytes.len();
        let mut bytes_vec = bytes.to_vec();
        bytes_vec.resize(ADDRESS_LENGTH, 0);

        let mut bytes = [0u8; ADDRESS_LENGTH];
        bytes.copy_from_slice(bytes_vec.as_slice());
        Address(bytes, len as u8)
    }
}

impl Into<Addr> for Address {
    fn into(self) -> Addr {
        Addr::unchecked(self.to_string())
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        let significant_bytes = &self.0[0..self.1 as usize];
        String::from_utf8(significant_bytes.to_vec()).unwrap()
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = hex::encode(self.to_string());
        f.debug_struct("Address").field("data", &name).finish()
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "null" {
            return Err(String::from("Null address"))
        }
        let bytes = s.as_bytes();
        Ok(Address::new(bytes))
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let str: String = self.to_string();
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
    use cosmwasm_std::Addr;

    use crate::Address;

    #[test]
    fn serde() {
        let address_str = "juno1sh3lp27j2xkpgj46qszltfv9nm7ewdnzy724tgm5u8ze2wklrekqn99vrs";
        let address = Address::new(address_str.as_bytes());

        let serialized_address = serde_json_wasm::to_vec(&address).unwrap();
        let deserialized_address: Address = serde_json_wasm::from_slice(&serialized_address).unwrap();

        assert_eq!(address, deserialized_address);

        let expected_address = Addr::unchecked(address_str);
        let actual_address: Addr = deserialized_address.into();

        assert_eq!(actual_address, expected_address);
    }
}
