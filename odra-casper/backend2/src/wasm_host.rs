extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use casper_contract::contract_api::{runtime, storage};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use casper_event_standard::{Schema, Schemas};
use casper_types::system::CallStackElement;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    ApiError, CLTyped, ContractPackageHash, EntryPoints, Key, URef
};
use odra_types::{Address, ExecutionError};

use crate::consts;

lazy_static::lazy_static! {
    static ref STATE: URef = {
        let key = runtime::get_key(consts::STATE_KEY).unwrap_or_revert();
        let state_uref: URef = *key.as_uref().unwrap_or_revert();
        state_uref
    };

    static ref STATE_BYTES: Vec<u8> = {
        (*STATE).into_bytes().unwrap_or_revert()
    };
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
        revert(ExecutionError::contract_already_installed().code()); // TODO: fix
    };

    // Parse events.
    let mut schemas = Schemas::new();
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

fn initial_named_keys(schemas: Schemas) -> NamedKeys {
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

/// Revert the execution.
#[inline(always)]
pub fn revert(error: u16) -> ! {
    runtime::revert(ApiError::User(error))
}

pub fn get_value(key: &[u8]) -> Option<Vec<u8>> {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_ptr = key.as_ptr();
    let dictionary_item_key_size = key.len();

    let value_size = {
        let mut value_size = core::mem::MaybeUninit::uninit();
        let ret = unsafe {
            casper_contract::ext_ffi::casper_dictionary_get(
                uref_ptr,
                uref_size,
                dictionary_item_key_ptr,
                dictionary_item_key_size,
                value_size.as_mut_ptr()
            )
        };
        match casper_types::api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    let value_bytes = Vec::from_bytes(value_bytes.as_slice()).unwrap_or_revert();
    Some(value_bytes.0)
}

fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = casper_contract::contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = core::mem::MaybeUninit::uninit();
    let ret = unsafe {
        casper_contract::ext_ffi::casper_read_host_buffer(
            dest.as_mut_ptr(),
            dest.len(),
            bytes_written.as_mut_ptr()
        )
    };
    casper_types::api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

pub fn set_value(key: &[u8], value: &[u8]) {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_ptr = key.as_ptr();
    let dictionary_item_key_size = key.len();

    let cl_value = casper_types::CLValue::from_t(value.to_vec()).unwrap_or_revert();
    let (value_ptr, value_size, _bytes) = to_ptr(cl_value);
    // let value_ptr = value.as_ptr();
    // let value_size = value.len();

    let result = unsafe {
        let ret = casper_contract::ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            value_ptr,
            value_size
        );
        casper_types::api_error::result_from(ret)
    };

    result.unwrap_or_revert();
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

/// Gets the immediate session caller of the current execution.
///
/// This function ensures that only session code can execute this function, and disallows stored
/// session/stored contracts.
#[inline(always)]
pub fn caller() -> Address {
    let second_elem = take_call_stack_elem(1);
    call_stack_element_to_address(second_elem)
}

/// Gets the address of the currently run contract
#[inline(always)]
pub fn self_address() -> Address {
    let first_elem = take_call_stack_elem(0);
    call_stack_element_to_address(first_elem)
}

/// Returns address based on a [`CallStackElement`].
///
/// For `Session` and `StoredSession` variants it will return account hash, and for `StoredContract`
/// case it will use contract hash as the address.
fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Address {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Address::try_from(account_hash)
            .map_err(|e| ApiError::User(ExecutionError::from(e).code()))
            .unwrap_or_revert(),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session
            // wants to interact, caller's address will be used.
            Address::try_from(account_hash)
                .map_err(|e| ApiError::User(ExecutionError::from(e).code()))
                .unwrap_or_revert()
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Address::try_from(contract_package_hash)
            .map_err(|e| ApiError::User(ExecutionError::from(e).code()))
            .unwrap_or_revert()
    }
}

fn take_call_stack_elem(n: usize) -> CallStackElement {
    runtime::get_call_stack()
        .into_iter()
        .nth_back(n)
        .unwrap_or_revert()
}
