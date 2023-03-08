//! Better address representation for Casper.

use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    CLType, CLTyped, ContractPackageHash, Key
};
use odra_types::address::OdraAddress;
use odra_types::AddressError;
use odra_types::AddressError::ZeroAddress;

/// An enum representing an [`AccountHash`] or a [`ContractPackageHash`].
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum CasperAddress {
    /// Represents an account hash.
    Account(AccountHash),
    /// Represents a contract package hash.
    Contract(ContractPackageHash)
}

impl CasperAddress {
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
}

impl OdraAddress for CasperAddress {
    fn is_contract(&self) -> bool {
        self.as_contract_package_hash().is_some()
    }
}

impl TryFrom<ContractPackageHash> for CasperAddress {
    type Error = AddressError;
    fn try_from(contract_package_hash: ContractPackageHash) -> Result<Self, Self::Error> {
        if contract_package_hash.value().iter().all(|&b| b == 0) {
            return Err(ZeroAddress);
        }
        Ok(Self::Contract(contract_package_hash))
    }
}

impl TryFrom<AccountHash> for CasperAddress {
    type Error = AddressError;
    fn try_from(account_hash: AccountHash) -> Result<Self, Self::Error> {
        if account_hash.value().iter().all(|&b| b == 0) {
            return Err(ZeroAddress);
        }
        Ok(Self::Account(account_hash))
    }
}

impl From<CasperAddress> for Key {
    fn from(address: CasperAddress) -> Self {
        match address {
            CasperAddress::Account(account_hash) => Key::Account(account_hash),
            CasperAddress::Contract(contract_package_hash) => {
                Key::Hash(contract_package_hash.value())
            }
        }
    }
}

impl TryFrom<Key> for CasperAddress {
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

impl CLTyped for CasperAddress {
    fn cl_type() -> CLType {
        CLType::Key
    }
}

impl ToBytes for CasperAddress {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Key::from(*self).to_bytes()
    }

    fn serialized_length(&self) -> usize {
        Key::from(*self).serialized_length()
    }
}

impl FromBytes for CasperAddress {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (key, remainder) = Key::from_bytes(bytes)?;

        let address = match key {
            Key::Account(account_hash) => CasperAddress::Account(account_hash),
            Key::Hash(raw_contract_package_hash) => {
                CasperAddress::Contract(ContractPackageHash::new(raw_contract_package_hash))
            }
            _ => return Err(bytesrepr::Error::Formatting)
        };

        Ok((address, remainder))
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for CasperAddress {
    type Error = AddressError;
    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        let address = CasperAddress::from_bytes(value)
            .map(|(address, _)| address)
            .map_err(|_| AddressError::AddressCreationError)?;
        if address.to_bytes().unwrap().iter().all(|&x| x == 0) {
            Err(ZeroAddress)
        } else {
            Ok(address)
        }
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

    fn mock_account_hash() -> AccountHash {
        AccountHash::from_formatted_str(ACCOUNT_HASH).unwrap()
    }

    fn mock_contract_package_hash() -> ContractPackageHash {
        ContractPackageHash::from_formatted_str(CONTRACT_PACKAGE_HASH).unwrap()
    }

    #[test]
    fn test_casper_address_account_hash_conversion() {
        let account_hash = mock_account_hash();

        // It is possible to convert CasperAddress back to AccountHash.
        let casper_address = CasperAddress::try_from(account_hash).unwrap();
        assert_eq!(casper_address.as_account_hash().unwrap(), &account_hash);

        // It is not possible to convert CasperAddress to ContractPackageHash.
        assert!(casper_address.as_contract_package_hash().is_none());

        // And it is not a contract.
        assert!(!casper_address.is_contract());

        test_casper_address_conversions(casper_address);
    }

    #[test]
    fn test_casper_address_contract_package_hash_conversion() {
        let contract_package_hash = mock_contract_package_hash();
        let casper_address = CasperAddress::try_from(contract_package_hash).unwrap();

        // It is possible to convert CasperAddress back to ContractPackageHash.
        assert_eq!(
            casper_address.as_contract_package_hash().unwrap(),
            &contract_package_hash
        );

        // It is not possible to convert CasperAddress to AccountHash.
        assert!(casper_address.as_account_hash().is_none());

        // And it is a contract.
        assert!(casper_address.is_contract());

        test_casper_address_conversions(casper_address);
    }

    fn test_casper_address_conversions(casper_address: CasperAddress) {
        // It can be converted into a Key and back to CasperAddress.
        let key = Key::from(casper_address);
        let restored = CasperAddress::try_from(key);
        assert_eq!(restored.unwrap(), casper_address);

        // It can be converted into bytes and back.
        let bytes = casper_address.to_bytes().unwrap();
        let (restored, rest) = CasperAddress::from_bytes(&bytes).unwrap();
        assert!(rest.is_empty());
        assert_eq!(restored, casper_address);
    }
}
