//! Casper backend for WASM.
//!
//! It provides all the required functions to communicate between Odra and Casper.
use casper_contract::{
    contract_api::system::transfer_from_purse_to_account, unwrap_or_revert::UnwrapOrRevert
};
use casper_types::U512;
use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_casper_types::{Address, Balance, BlockTime, CallArgs, OdraType};
use odra_types::{event::OdraEvent, ExecutionError};
use std::ops::Deref;

use crate::{casper_env, utils::get_main_purse};

pub(crate) static mut ATTACHED_VALUE: U512 = U512::zero();

/// Returns blocktime.
pub fn get_block_time() -> BlockTime {
    casper_env::get_block_time()
}

/// Returns contract caller.
pub fn caller() -> Address {
    casper_env::caller()
}

/// Returns current contract address.
pub fn self_address() -> Address {
    casper_env::self_address()
}

/// Store a value into the storage.
pub fn set_var<T: OdraType>(key: &str, value: T) {
    casper_env::set_key(key, value);
}

/// Read value from the storage.
pub fn get_var<T: OdraType>(key: &str) -> Option<T> {
    casper_env::get_key(key)
}

/// Store the mapping value under a given key.
pub fn set_dict_value<K: OdraType, V: OdraType>(dict: &str, key: &K, value: V) {
    casper_env::set_dict_value(dict, key, value);
}

/// Read value from the mapping.
pub fn get_dict_value<K: OdraType, T: OdraType>(dict: &str, key: &K) -> Option<T> {
    casper_env::get_dict_value(dict, key)
}

/// Revert the execution.
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>
{
    casper_env::revert(error.into().code());
}

/// Emits event.
pub fn emit_event<T>(event: T)
where
    T: OdraType + OdraEvent
{
    casper_env::emit_event(event);
}

/// Call another contract.
pub fn call_contract<T: OdraType>(
    address: Address,
    entrypoint: &str,
    args: CallArgs,
    amount: Option<Balance>
) -> T {
    let contract_package_hash = *address.as_contract_package_hash().unwrap_or_revert();
    if let Some(amount) = amount {
        casper_env::call_contract_with_amount(
            contract_package_hash,
            entrypoint,
            args.deref().clone(),
            amount.inner()
        )
    } else {
        casper_env::call_contract(contract_package_hash, entrypoint, args.deref().clone())
    }
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> Balance {
    Balance::from(1_000_000_000)
}

/// Returns the balance of the account associated with the currently executing contract.
pub fn self_balance() -> Balance {
    casper_env::self_balance().into()
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> Balance {
    unsafe { ATTACHED_VALUE.into() }
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens<B: Into<Balance>>(to: Address, amount: B) {
    let main_purse = get_main_purse();

    match to {
        Address::Account(account) => {
            transfer_from_purse_to_account(main_purse, account, amount.into().inner(), None)
                .unwrap_or_revert();
        }
        Address::Contract(_) => revert(ExecutionError::can_not_transfer_to_contract())
    };
}

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}
