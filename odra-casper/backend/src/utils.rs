//! A set of utility functions encapsulating some common interactions with the current runtime.

use casper_contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert
};
use casper_event_standard::Schema;
use casper_types::{CLType, ContractPackageHash, EntryPoints, URef, U512};
use odra_casper_shared::consts;
use odra_casper_types::Balance;
use odra_types::ExecutionError;

use crate::{
    casper_env,
    contract_env::{revert, ATTACHED_VALUE}
};

pub fn add_contract_version(
    contract_package_hash: ContractPackageHash,
    entry_points: EntryPoints,
    events: Vec<(String, Schema)>
) {
    casper_env::add_contract_version(contract_package_hash, entry_points, events);
}

/// Checks if given named argument exists.
pub fn named_arg_exists(name: &str) -> bool {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        casper_contract::ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize
        )
    };
    casper_types::api_error::result_from(ret).is_ok()
}

/// Gets the currently executing contract main purse [URef].
pub fn get_main_purse() -> URef {
    casper_env::get_or_create_purse()
}

/// Stores in memory the amount attached to the current call.
pub fn set_attached_value(amount: Balance) {
    unsafe {
        ATTACHED_VALUE = amount.inner();
    }
}

/// Zeroes the amount attached to the current call.
pub fn clear_attached_value() {
    unsafe { ATTACHED_VALUE = U512::zero() }
}

/// Transfers attached value to the currently executing contract.
pub fn handle_attached_value() {
    if named_arg_exists(consts::CARGO_PURSE_ARG) {
        let cargo_purse = runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = system::get_purse_balance(cargo_purse);
        if let Some(amount) = amount {
            let contract_purse = get_main_purse();
            system::transfer_from_purse_to_purse(cargo_purse, contract_purse, amount, None)
                .unwrap_or_revert();
            set_attached_value(amount.into());
        }
    }
}

/// Reverts with an [ExecutionError] if some value is attached to the call.
pub fn assert_no_attached_value() {
    if named_arg_exists(consts::CARGO_PURSE_ARG) {
        let cargo_purse = runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = casper_contract::contract_api::system::get_purse_balance(cargo_purse);
        if amount.is_some() && !amount.unwrap().is_zero() {
            revert(ExecutionError::non_payable());
        }
    }
}

pub fn non_reentrant_before() {
    let status: bool = casper_env::get_key(consts::REENTRANCY_GUARD).unwrap_or_default();
    if status {
        revert(ExecutionError::reentrant_call())
    };
    casper_env::set_key(consts::REENTRANCY_GUARD, true);
}

pub fn non_reentrant_after() {
    casper_env::set_key(consts::REENTRANCY_GUARD, false);
}

pub fn build_event(name: &str, fields: Vec<(&str, CLType)>) -> (String, Schema) {
    let mut s = Schema::new();
    fields.iter().for_each(|(name, cl_type)| {
        s.with_elem(name, cl_type.clone());
    });
    (name.to_owned(), s)
}
