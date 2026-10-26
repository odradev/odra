use odra_core::casper_types::contracts::ContractPackageHash;
use odra_core::casper_types::{account::AccountHash, PackageHash};
use odra_core::prelude::*;

#[cfg(test)]
pub fn account_address_from_str(str: &str) -> Address {
    use odra_core::casper_types::account::{
        ACCOUNT_HASH_FORMATTED_STRING_PREFIX, ACCOUNT_HASH_LENGTH
    };
    let desired_length = ACCOUNT_HASH_LENGTH * 2;
    let padding_length = desired_length - str.len();
    let padding = "0".repeat(padding_length);

    let account_str = format!("{}{}{}", ACCOUNT_HASH_FORMATTED_STRING_PREFIX, str, padding);
    Address::Account(AccountHash::from_formatted_str(account_str.as_str()).unwrap())
}

/// The address of the `i`-th contract deployed on OdraVM: `i` in decimal digits, padded with zeros on
/// the left (on the right, the 10th contract would get the address of the 1st).
pub fn contract_address_from_u32(i: u32) -> Address {
    use odra_core::casper_types::KEY_HASH_LENGTH;
    let account_str = format!("package-{:0>width$}", i, width = KEY_HASH_LENGTH * 2);
    let ph = PackageHash::from_formatted_str(account_str.as_str()).unwrap();
    let cph = ContractPackageHash::new(ph.value());
    Address::Contract(cph)
}

#[cfg(test)]
mod tests {
    use super::contract_address_from_u32;

    #[test]
    fn contract_addresses_are_unique() {
        let addresses = (1..=100)
            .map(contract_address_from_u32)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(addresses.len(), 100);
    }
}
