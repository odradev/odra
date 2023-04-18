//! A set of utility functions encapsulating some common interactions with the current runtime.

use casper_contract::contract_api::runtime;
use casper_event_standard::Schema;
use casper_types::{ContractPackageHash, EntryPoints, URef, U512, CLType};
use odra_casper_shared::consts;
use odra_casper_types::Balance;
use odra_types::ExecutionError;

use crate::{
    casper_env,
    contract_env::{revert, ATTACHED_VALUE}
};

pub fn add_contract_version(contract_package_hash: ContractPackageHash, entry_points: EntryPoints) {
    casper_env::add_contract_version(contract_package_hash, entry_points);
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
        let cargo_purse =
            casper_contract::contract_api::runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = casper_contract::contract_api::system::get_purse_balance(cargo_purse);
        if let Some(amount) = amount {
            let contract_purse = get_main_purse();
            let _ = casper_contract::contract_api::system::transfer_from_purse_to_purse(
                cargo_purse,
                contract_purse,
                amount,
                None
            );
            set_attached_value(amount.into());
        }
    }
}

/// Reverts with an [ExecutionError] if some value is attached to the call.
pub fn assert_no_attached_value() {
    if named_arg_exists(consts::CARGO_PURSE_ARG) {
        let cargo_purse =
            casper_contract::contract_api::runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = casper_contract::contract_api::system::get_purse_balance(cargo_purse);
        if amount.is_some() && !amount.unwrap().is_zero() {
            revert(ExecutionError::non_payable());
        }
    }
}


pub fn register_events(events: Vec<(String, Schema)>) {
    let mut schemas = casper_event_standard::Schemas::new();
    events.iter().for_each(|(name, schema)| {
        schemas.0.insert(name.to_owned(), schema.clone());
    });
    
    casper_event_standard::init(schemas);
    runtime::remove_key(casper_event_standard::EVENTS_DICT);
    runtime::remove_key(casper_event_standard::EVENTS_LENGTH);
    runtime::remove_key(casper_event_standard::EVENTS_SCHEMA);
    runtime::remove_key(casper_event_standard::CES_VERSION);
    runtime::remove_key(casper_event_standard::CES_VERSION_KEY);
}

pub fn build_event(name: &str, fields: Vec<(&str, CLType)>) -> (String, Schema) {
    let mut s = Schema::new();
    fields.iter().for_each(|(name, cl_type)| {
        s.with_elem(name, cl_type.clone());
    });
    (name.to_owned(), s)
}
