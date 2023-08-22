//! A set of utility functions encapsulating some common interactions with the current runtime.

use alloc::{format, string::String, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert
};
use casper_event_standard::Schema;
use casper_types::{
    contracts::NamedKeys, CLType, ContractPackageHash, EntryPoints, Key, URef, U512
};
use odra_casper_shared::consts;
use odra_casper_types::Balance;
use odra_types::ExecutionError;

use crate::{
    casper_env,
    contract_env::{revert, ATTACHED_VALUE}
};

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
    ret == 0
}

/// Gets the currently executing contract main purse [URef].
/// It creates a new purse if it doesn't exist.
pub fn get_or_create_main_purse() -> URef {
    casper_env::get_or_create_main_purse()
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
    // If the cargo purse argument is not present, do nothing.
    // Attached value is set to zero by default.
    if !named_arg_exists(consts::CARGO_PURSE_ARG) {
        return;
    }

    // Handle attached value.
    let cargo_purse = runtime::get_named_arg(consts::CARGO_PURSE_ARG);
    let amount = system::get_purse_balance(cargo_purse);
    if let Some(amount) = amount {
        let contract_purse = get_or_create_main_purse();
        system::transfer_from_purse_to_purse(cargo_purse, contract_purse, amount, None)
            .unwrap_or_revert();
        set_attached_value(amount.into());
    } else {
        revert(ExecutionError::native_token_transfer_error())
    }
}

pub fn non_reentrant_before() {
    let status: bool = casper_env::get_key(&consts::REENTRANCY_GUARD).unwrap_or_default();
    if status {
        revert(ExecutionError::reentrant_call())
    };
    casper_env::set_key(&consts::REENTRANCY_GUARD, true);
}

pub fn non_reentrant_after() {
    casper_env::set_key(&consts::REENTRANCY_GUARD, false);
}

pub fn build_event(name: String, fields: Vec<(&'static str, CLType)>) -> (String, Schema) {
    let mut schema = Schema::new();
    for (name, cl_type) in fields {
        schema.with_elem(name, cl_type);
    }
    (name, schema)
}

pub fn install_contract(
    entry_points: EntryPoints,
    events: Vec<(String, Schema)>
) -> ContractPackageHash {
    // Read arguments
    let package_hash_key: String = runtime::get_named_arg(consts::PACKAGE_HASH_KEY_NAME_ARG);
    let allow_key_override: bool = runtime::get_named_arg(consts::ALLOW_KEY_OVERRIDE_ARG);
    let is_upgradable: bool = runtime::get_named_arg(consts::IS_UPGRADABLE_ARG);

    // Check if the package hash is already in the storage.
    // Revert if key override is not allowed.
    if !allow_key_override && runtime::has_key(&package_hash_key) {
        revert(ExecutionError::contract_already_installed());
    };

    // Parse events.
    let mut schemas = casper_event_standard::Schemas::new();
    for (name, schema) in events {
        schemas.0.insert(name, schema);
    }

    // Prepare named keys.
    let named_keys = initial_named_keys(schemas);

    // Create new contract.
    if is_upgradable {
        let access_uref_key = format!("{}_access_token", package_hash_key);
        storage::new_contract(
            entry_points,
            Some(named_keys),
            Some(package_hash_key.clone()),
            Some(access_uref_key)
        );
    } else {
        storage::new_locked_contract(
            entry_points,
            Some(named_keys),
            Some(package_hash_key.clone()),
            None
        );
    }

    // Read contract package hash from the storage.
    runtime::get_key(&package_hash_key)
        .unwrap_or_revert()
        .into_hash()
        .unwrap_or_revert()
        .into()
}

fn initial_named_keys(schemas: casper_event_standard::Schemas) -> NamedKeys {
    let mut named_keys = casper_types::contracts::NamedKeys::new();
    named_keys.insert(
        String::from(consts::STATE_KEY),
        Key::URef(storage::new_dictionary(consts::STATE_KEY).unwrap_or_revert())
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_DICT),
        Key::URef(storage::new_dictionary(casper_event_standard::EVENTS_DICT).unwrap_or_revert())
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_LENGTH),
        Key::URef(storage::new_uref(0u32))
    );
    named_keys.insert(
        String::from(casper_event_standard::CES_VERSION_KEY),
        Key::URef(storage::new_uref(casper_event_standard::CES_VERSION))
    );
    named_keys.insert(
        String::from(casper_event_standard::EVENTS_SCHEMA),
        Key::URef(storage::new_uref(schemas))
    );

    runtime::remove_key(consts::STATE_KEY);
    runtime::remove_key(casper_event_standard::EVENTS_DICT);
    runtime::remove_key(casper_event_standard::EVENTS_LENGTH);
    runtime::remove_key(casper_event_standard::EVENTS_SCHEMA);
    runtime::remove_key(casper_event_standard::CES_VERSION_KEY);

    named_keys
}

pub fn create_constructor_group(contract_package_hash: ContractPackageHash) -> URef {
    storage::create_contract_user_group(
        contract_package_hash,
        consts::CONSTRUCTOR_GROUP_NAME,
        1,
        Default::default()
    )
    .unwrap_or_revert()
    .pop()
    .unwrap_or_revert()
}

pub fn revoke_access_to_constructor_group(
    contract_package_hash: ContractPackageHash,
    constructor_access: URef
) {
    let mut urefs = alloc::collections::BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(
        contract_package_hash,
        consts::CONSTRUCTOR_GROUP_NAME,
        urefs
    )
    .unwrap_or_revert();
}

pub fn load_constructor_name_arg() -> String {
    runtime::get_named_arg(consts::CONSTRUCTOR_NAME_ARG)
}

pub fn revert_on_unknown_constructor() {
    revert(ExecutionError::unknown_constructor())
}
