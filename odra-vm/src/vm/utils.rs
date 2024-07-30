use odra_core::casper_types::PackageHash;
use odra_core::{casper_types::account::AccountHash, Address};

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

pub fn contract_address_from_u32(i: u32) -> Address {
    use odra_core::casper_types::KEY_HASH_LENGTH;
    let desired_length = KEY_HASH_LENGTH * 2;
    let padding_length = desired_length - i.to_string().len();
    let padding = "0".repeat(padding_length);

    let a = i.to_string();
    let account_str = format!("{}{}{}", "package-", a, padding);
    Address::Contract(PackageHash::from_formatted_str(account_str.as_str()).unwrap())
}
