//! Better address representation for Casper.
use crate::prelude::*;
use crate::{AddressError, VmError};
use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractPackageHash, Key, PublicKey
};
use serde::{Deserialize, Serialize};

/// The length of the hash part of an address.
const ADDRESS_HASH_LENGTH: usize = 64;
/// An address has format `hash-<64-byte-hash>`.
const CONTRACT_STR_LENGTH: usize = 69;
/// An address has format `contract-package-wasm<64-byte-hash>`.
const LEGACY_CONTRACT_STR_LENGTH: usize = 85;
/// An address has format `account-hash-<64-byte-hash>`.
const ACCOUNT_STR_LENGTH: usize = 77;

/// An enum representing an [`AccountHash`] or a [`ContractPackageHash`].
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Address {
    /// Represents an account hash.
    Account(AccountHash),
    /// Represents a contract package hash.
    Contract(ContractPackageHash)
}

/// A trait for types that can be converted into an [`Address`].
pub trait Addressable {
    /// Returns a reference to the [`Address`] of the type.
    fn address(&self) -> &Address;
}

impl Addressable for Address {
    fn address(&self) -> &Address {
        self
    }
}

impl Address {
    /// Creates a new `Address` from a hex-encoded string.
    pub const fn new(input: &'static str) -> OdraResult<Self> {
        let src: &[u8] = input.as_bytes();
        let src_len: usize = src.len();

        if let Ok(dst) = decode_base16(src) {
            // depending on the length of the input, we can determine the type of address
            match src_len {
                LEGACY_CONTRACT_STR_LENGTH => Ok(Self::Contract(ContractPackageHash::new(dst))),
                ACCOUNT_STR_LENGTH => Ok(Self::Account(AccountHash::new(dst))),
                CONTRACT_STR_LENGTH => Ok(Self::Contract(ContractPackageHash::new(dst))),
                _ => Err(OdraError::ExecutionError(
                    ExecutionError::AddressCreationFailed
                ))
            }
        } else {
            Err(OdraError::ExecutionError(
                ExecutionError::AddressCreationFailed
            ))
        }
    }

    /// Returns the inner account hash if `self` is the `Account` variant.
    pub fn as_account_hash(&self) -> Option<&AccountHash> {
        if let Self::Account(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner contract hash if `self` is the `Contract` variant.
    pub fn as_contract_package_hash(&self) -> Option<&ContractPackageHash> {
        if let Self::Contract(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns true if the address is a contract address.
    pub fn is_contract(&self) -> bool {
        self.as_contract_package_hash().is_some()
    }
}

impl TryFrom<ContractPackageHash> for Address {
    type Error = AddressError;
    fn try_from(contract_package_hash: ContractPackageHash) -> Result<Self, Self::Error> {
        if contract_package_hash.value().iter().all(|&b| b == 0) {
            return Err(AddressError::ZeroAddress);
        }
        Ok(Self::Contract(contract_package_hash))
    }
}

impl TryFrom<AccountHash> for Address {
    type Error = AddressError;
    fn try_from(account_hash: AccountHash) -> Result<Self, Self::Error> {
        if account_hash.value().iter().all(|&b| b == 0) {
            return Err(AddressError::ZeroAddress);
        }
        Ok(Self::Account(account_hash))
    }
}

impl From<Address> for Key {
    fn from(address: Address) -> Self {
        match address {
            Address::Account(account_hash) => Key::Account(account_hash),
            Address::Contract(contract_package_hash) => Key::Hash(contract_package_hash.value())
        }
    }
}

impl TryFrom<Key> for Address {
    type Error = AddressError;

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        match key {
            Key::Account(account_hash) => Self::try_from(account_hash),
            Key::Hash(contract_package_hash) => {
                Self::try_from(ContractPackageHash::new(contract_package_hash))
            }
            _ => Err(AddressError::AddressCreationError)
        }
    }
}

impl From<PublicKey> for Address {
    fn from(public_key: PublicKey) -> Self {
        Self::Account(public_key.to_account_hash())
    }
}

impl CLTyped for Address {
    fn cl_type() -> CLType {
        CLType::Key
    }
}

impl ToBytes for Address {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Key::from(*self).to_bytes()
    }

    fn serialized_length(&self) -> usize {
        Key::from(*self).serialized_length()
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (key, remainder) = Key::from_bytes(bytes)?;

        let address = match key {
            Key::Account(account_hash) => Address::Account(account_hash),
            Key::Hash(raw_contract_package_hash) => {
                Address::Contract(ContractPackageHash::new(raw_contract_package_hash))
            }
            _ => return Err(bytesrepr::Error::Formatting)
        };

        Ok((address, remainder))
    }
}

impl TryFrom<&[u8; 33]> for Address {
    type Error = AddressError;
    fn try_from(value: &[u8; 33]) -> Result<Self, Self::Error> {
        let address = Address::from_bytes(value)
            .map(|(address, _)| address)
            .map_err(|_| AddressError::AddressCreationError)?;
        if address
            .to_bytes()
            .map_err(|_| AddressError::AddressCreationError)?
            .iter()
            .all(|&x| x == 0)
        {
            Err(AddressError::ZeroAddress)
        } else {
            Ok(address)
        }
    }
}

impl FromStr for Address {
    type Err = OdraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Key::from_formatted_str(s) {
            Err(_) => Err(OdraError::VmError(VmError::Deserialization)),
            Ok(key) => match key {
                Key::Account(_) | Key::Hash(_) => match key.try_into() {
                    Ok(address) => Ok(address),
                    Err(_) => Err(OdraError::VmError(VmError::Deserialization))
                },
                _ => Err(OdraError::VmError(VmError::Deserialization))
            }
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for Address {
    fn to_string(&self) -> String {
        Key::from(*self).to_formatted_string()
    }
}

impl Serialize for Address {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Address::from_str(&s).map_err(|_| serde::de::Error::custom("Address deserialization error"))
    }
}

const fn decode_base16(input: &[u8]) -> Result<[u8; 32], &'static str> {
    // fail fast if the input is too short
    let input_len = input.len();
    if input_len < ADDRESS_HASH_LENGTH {
        return Err("Input too short");
    }
    // An address is always 32 bytes long.
    let mut output = [0u8; 32];
    let mut i = 0;
    let mut j = 0;
    // In a const fn, we can't use a for loop.
    // We consider only the last 64 characters of the input.
    while i < 64 {
        let high_value = match hex_char_to_value(input[input_len - ADDRESS_HASH_LENGTH + i]) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        let low_value = match hex_char_to_value(input[input_len - ADDRESS_HASH_LENGTH + i + 1]) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        output[j] = (high_value << 4) | low_value;
        i += 2;
        j += 1;
    }

    Ok(output)
}

const fn hex_char_to_value(c: u8) -> Result<u8, &'static str> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err("Invalid character in input")
    }
}

#[cfg(test)]
mod tests {
    use casper_types::EraId;

    use super::*;

    // TODO: casper-types > 1.5.0 will have prefix fixed.
    const CONTRACT_PACKAGE_HASH: &str =
        "contract-package-wasm7ba9daac84bebee8111c186588f21ebca35550b6cf1244e71768bd871938be6a";
    const ACCOUNT_HASH: &str =
        "account-hash-3b4ffcfb21411ced5fc1560c3f6ffed86f4885e5ea05cde49d90962a48a14d95";
    const CONTRACT_HASH: &str =
        "hash-7ba9daac84bebee8111c186588f21ebca35550b6cf1244e71768bd871938be6a";

    fn mock_account_hash() -> AccountHash {
        AccountHash::from_formatted_str(ACCOUNT_HASH).unwrap()
    }

    fn mock_contract_package_hash() -> ContractPackageHash {
        ContractPackageHash::from_formatted_str(CONTRACT_PACKAGE_HASH).unwrap()
    }

    #[test]
    fn test_casper_address_new() {
        let address = Address::new(CONTRACT_PACKAGE_HASH).unwrap();
        assert!(address.is_contract());
        assert_eq!(
            address.as_contract_package_hash().unwrap(),
            &mock_contract_package_hash()
        );

        let address = Address::new(ACCOUNT_HASH).unwrap();
        assert!(!address.is_contract());
        assert_eq!(address.as_account_hash().unwrap(), &mock_account_hash());

        let address = Address::new(CONTRACT_HASH).unwrap();
        assert!(address.is_contract());
    }

    #[test]
    fn contract_package_hash_from_str() {
        let valid_prefix =
            "account-hash-0000000000000000000000000000000000000000000000000000000000000000";
        assert!(Address::new(valid_prefix).is_ok());

        let invalid_prefix =
            "account-hash0000000000000000000000000000000000000000000000000000000000000000";
        assert!(Address::new(invalid_prefix).is_err());

        let short_addr =
            "account-hash-00000000000000000000000000000000000000000000000000000000000000";
        assert!(Address::new(short_addr).is_err());

        let long_addr =
            "account-hash-000000000000000000000000000000000000000000000000000000000000000000";
        assert!(Address::new(long_addr).is_err());

        let invalid_hex =
            "account-hash-000000000000000000000000000000000000000000000000000000000000000g";
        assert!(Address::new(invalid_hex).is_err());
    }

    #[test]
    fn test_casper_address_account_hash_conversion() {
        let account_hash = mock_account_hash();

        // It is possible to convert Address back to AccountHash.
        let casper_address = Address::try_from(account_hash).unwrap();
        assert_eq!(casper_address.as_account_hash().unwrap(), &account_hash);

        // It is not possible to convert Address to ContractPackageHash.
        assert!(casper_address.as_contract_package_hash().is_none());

        // And it is not a contract.
        assert!(!casper_address.is_contract());

        test_casper_address_conversions(casper_address);
    }

    #[test]
    fn test_casper_address_contract_package_hash_conversion() {
        let contract_package_hash = mock_contract_package_hash();
        let casper_address = Address::try_from(contract_package_hash).unwrap();

        // It is possible to convert Address back to ContractPackageHash.
        assert_eq!(
            casper_address.as_contract_package_hash().unwrap(),
            &contract_package_hash
        );

        // It is not possible to convert Address to AccountHash.
        assert!(casper_address.as_account_hash().is_none());

        // And it is a contract.
        assert!(casper_address.is_contract());

        test_casper_address_conversions(casper_address);
    }

    fn test_casper_address_conversions(casper_address: Address) {
        // It can be converted into a Key and back to Address.
        let key = Key::from(casper_address);
        let restored = Address::try_from(key);
        assert_eq!(restored.unwrap(), casper_address);

        // It can be converted into bytes and back.
        let bytes = casper_address.to_bytes().unwrap();
        let (restored, rest) = Address::from_bytes(&bytes).unwrap();
        assert!(rest.is_empty());
        assert_eq!(restored, casper_address);
    }

    #[test]
    fn test_casper_address_from_to_string() {
        let address = Address::from_str(CONTRACT_HASH).unwrap();
        assert!(address.is_contract());
        assert_eq!(&address.to_string(), CONTRACT_HASH);

        let address = Address::from_str(ACCOUNT_HASH).unwrap();
        assert!(!address.is_contract());
        assert_eq!(&address.to_string(), ACCOUNT_HASH);

        assert_eq!(
            Address::from_str(CONTRACT_PACKAGE_HASH).unwrap_err(),
            OdraError::VmError(VmError::Deserialization)
        )
    }

    #[test]
    fn test_from_key_fails() {
        let key = Key::EraInfo(EraId::from(42));
        assert_eq!(
            Address::try_from(key).unwrap_err(),
            AddressError::AddressCreationError
        );
    }

    #[test]
    fn test_address_serde_roundtrip() {
        // Test Account serialization.
        let address_json = format!("\"{}\"", ACCOUNT_HASH);
        let address = Address::from_str(ACCOUNT_HASH).unwrap();
        let serialized = serde_json::to_string(&address).unwrap();
        assert_eq!(serialized, address_json);

        // Test Account deserialization.
        let deserialized: Address = serde_json::from_str(&address_json).unwrap();
        assert_eq!(deserialized, address);

        // Test Account serialization roundtrip.
        let serialized = serde_json::to_string(&address).unwrap();
        let deserialized: Address = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, address);

        // Test Contract serialization.
        let address_json = format!("\"{}\"", CONTRACT_HASH);
        let address = Address::from_str(CONTRACT_HASH).unwrap();
        let serialized = serde_json::to_string(&address).unwrap();
        assert_eq!(serialized, address_json);

        // Test Contract deserialization.
        let deserialized: Address = serde_json::from_str(&address_json).unwrap();
        assert_eq!(deserialized, address);

        // Test Contract serialization roundtrip.
        let serialized = serde_json::to_string(&address).unwrap();
        let deserialized: Address = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, address);
    }
}
