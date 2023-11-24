//! Better address representation for Casper.
use crate::prelude::*;
use crate::AddressError::ZeroAddress;
use crate::{AddressError, OdraError, VmError};
use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractPackageHash, Key, PublicKey
};

/// An enum representing an [`AccountHash`] or a [`ContractPackageHash`].
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Address {
    /// Represents an account hash.
    Account(AccountHash),
    /// Represents a contract package hash.
    Contract(ContractPackageHash)
}

impl Address {
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

    // TODO: move those methods to odra_vm as they shouldn't be public
    pub fn account_from_str(str: &str) -> Self {
        use casper_types::account::{ACCOUNT_HASH_FORMATTED_STRING_PREFIX, ACCOUNT_HASH_LENGTH};
        let desired_length = ACCOUNT_HASH_LENGTH * 2;
        let padding_length = desired_length - str.len();
        let padding = "0".repeat(padding_length);

        let account_str = format!("{}{}{}", ACCOUNT_HASH_FORMATTED_STRING_PREFIX, str, padding);
        Self::Account(AccountHash::from_formatted_str(account_str.as_str()).unwrap())
    }

    // TODO: move those methods to odra_vm as they shouldn't be public
    pub fn contract_from_u32(i: u32) -> Self {
        use casper_types::KEY_HASH_LENGTH;
        let desired_length = KEY_HASH_LENGTH * 2;
        let padding_length = desired_length - i.to_string().len();
        let padding = "0".repeat(padding_length);

        let a = i.to_string();
        let account_str = format!("{}{}{}", "contract-package-", a, padding);
        Self::Contract(ContractPackageHash::from_formatted_str(account_str.as_str()).unwrap())
    }
}

impl OdraAddress for Address {
    fn is_contract(&self) -> bool {
        self.as_contract_package_hash().is_some()
    }
}

impl TryFrom<ContractPackageHash> for Address {
    type Error = AddressError;
    fn try_from(contract_package_hash: ContractPackageHash) -> Result<Self, Self::Error> {
        if contract_package_hash.value().iter().all(|&b| b == 0) {
            return Err(ZeroAddress);
        }
        Ok(Self::Contract(contract_package_hash))
    }
}

impl TryFrom<AccountHash> for Address {
    type Error = AddressError;
    fn try_from(account_hash: AccountHash) -> Result<Self, Self::Error> {
        if account_hash.value().iter().all(|&b| b == 0) {
            return Err(ZeroAddress);
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
            Err(ZeroAddress)
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

impl ToString for Address {
    fn to_string(&self) -> String {
        Key::from(*self).to_formatted_string()
    }
}

#[cfg(test)]
mod tests {
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
}

pub trait OdraAddress {
    /// Returns true if the address is a contract address.
    fn is_contract(&self) -> bool;
}
