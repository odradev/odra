//! Functions that interact with the casper host environment.
//!
//! This module provides functions for interacting with the casper host environment, including
//! installing contracts, reverting contract execution, accessing named arguments, getting the
//! block time, performing cryptographic operations, manipulating contract storage, transferring
//! tokens, emitting events, and more.
//!
//! Build on top of the [casper_contract] crate.

use casper_contract::contract_api::storage::new_uref;
use casper_contract::{
    contract_api::{
        self, runtime, storage,
        system::{
            create_purse, get_purse_balance, transfer_from_purse_to_account,
            transfer_from_purse_to_purse
        }
    },
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert
};
use core::mem::MaybeUninit;
use odra_core::casper_types::bytesrepr::deserialize;
use odra_core::casper_types::{
    api_error, bytesrepr,
    bytesrepr::{Bytes, FromBytes, ToBytes},
    contracts::NamedKeys,
    system::CallStackElement,
    ApiError, CLTyped, CLValue, ContractPackageHash, ContractVersion, EntryPoints, Key,
    RuntimeArgs, URef, DICTIONARY_ITEM_KEY_MAX_LENGTH, U512, UREF_SERIALIZED_LENGTH
};
use odra_core::consts;
use odra_core::{
    args::EntrypointArgument,
    casper_event_standard::{self, Schema, Schemas}
};
use odra_core::{prelude::*, CallDef};

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

pub(crate) static mut ATTACHED_VALUE: U512 = U512::zero();

/// Installs a contract from a contract package.
///
/// Create a locked contract stored under a [Key::Hash]. The contract is upgradeable or not, depending on the
/// value of `odra_cfg_is_upgradable` argument.
///
/// If a contract with the same name already exists, it may be override depending on the value of `odra_cfg_allow_key_override`
/// argument.
///
/// Along with the contract, named keys with events and state are created.
pub fn install_contract(
    entry_points: EntryPoints,
    events: Schemas,
    init_args: Option<RuntimeArgs>
) -> ContractPackageHash {
    // Read arguments
    let package_hash_key: String = runtime::get_named_arg(consts::PACKAGE_HASH_KEY_NAME_ARG);
    let allow_key_override: bool = runtime::get_named_arg(consts::ALLOW_KEY_OVERRIDE_ARG);
    let is_upgradable: bool = runtime::get_named_arg(consts::IS_UPGRADABLE_ARG);

    // Check if the package hash is already in the storage.
    // Revert if key override is not allowed.
    if !allow_key_override && runtime::has_key(&package_hash_key) {
        revert(ExecutionError::ContractAlreadyInstalled.code()); // TODO: fix
    };

    // Prepare named keys.
    let named_keys = initial_named_keys(events);

    // Create new contract.
    let access_uref_key = format!("{}_access_token", package_hash_key);
    if is_upgradable {
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
            Some(access_uref_key)
        );
    }

    // Read contract package hash from the storage.
    let contract_package_hash: ContractPackageHash = runtime::get_key(&package_hash_key)
        .unwrap_or_revert()
        .into_hash()
        .unwrap_or_revert()
        .into();

    if let Some(args) = init_args {
        let init_access = create_constructor_group(contract_package_hash);
        let _: () = runtime::call_versioned_contract(contract_package_hash, None, "init", args);
        revoke_access_to_constructor_group(contract_package_hash, init_access);
    }

    contract_package_hash
}

/// Stops a contract execution and reverts the state with a given error.
#[inline(always)]
pub fn revert(error: u16) -> ! {
    runtime::revert(ApiError::User(error))
}

/// Returns given named argument passed to the host. The result is not deserialized,
/// is returned as a `Vec<u8>`.
pub fn get_named_arg(name: &str) -> Result<Vec<u8>, ApiError> {
    let arg_size = get_named_arg_size(name)?;
    if arg_size > 0 {
        let data_non_null_ptr = contract_api::alloc_bytes(arg_size);
        let ret = unsafe {
            ext_ffi::casper_get_named_arg(
                name.as_bytes().as_ptr(),
                name.len(),
                data_non_null_ptr.as_ptr(),
                arg_size
            )
        };
        if ret != 0 {
            return Err(ApiError::from(ret as u32));
        }
        unsafe {
            Ok(Vec::from_raw_parts(
                data_non_null_ptr.as_ptr(),
                arg_size,
                arg_size
            ))
        }
    } else {
        Ok(Vec::new())
    }
}

/// Gets the current block time.
#[inline(always)]
pub fn get_block_time() -> u64 {
    runtime::get_blocktime().into()
}

/// Hashes the given bytes using the BLAKE2b hash function.
#[inline(always)]
pub fn blake2b(input: &[u8]) -> [u8; 32] {
    runtime::blake2b(input)
}

/// Writes a value under a key to the contract's storage.
pub fn set_value(key: &[u8], value: &[u8]) {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_size = key.len();
    let dictionary_item_key_ptr = key.as_ptr();

    let cl_value = CLValue::from_t(value.to_vec()).unwrap_or_revert();
    let (value_ptr, value_size, _bytes) = to_ptr(cl_value);

    let result = unsafe {
        let ret = ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            value_ptr,
            value_size
        );
        api_error::result_from(ret)
    };

    result.unwrap_or_revert();
}

/// Gets a value under a key from the contract's storage.
pub fn get_value(key: &[u8]) -> Option<Vec<u8>> {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let dictionary_item_key_size = key.len();
    let dictionary_item_key_ptr = key.as_ptr();

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_dictionary_get(
                uref_ptr,
                uref_size,
                dictionary_item_key_ptr,
                dictionary_item_key_size,
                value_size.as_mut_ptr()
            )
        };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    let value_bytes = Vec::from_bytes(value_bytes.as_slice()).unwrap_or_revert();
    Some(value_bytes.0)
}

/// Writes a value under a named key to the contract's storage.
pub fn set_named_key(name: &str, value: CLValue) {
    match runtime::get_key(name) {
        Some(key) => {
            write(key, value);
        }
        None => {
            let new_uref = write_new(value);
            runtime::put_key(name, new_uref.into());
        }
    };
}

/// Gets a value under a named key from the contract's storage.
pub fn get_named_key(name: &str) -> Option<Bytes> {
    match runtime::get_key(name) {
        Some(key) => read(key),
        None => None
    }
}

/// Writes a value under a key in a dictionary to a contract's storage.
pub fn set_dictionary_value(dictionary_name: &str, key: &[u8], value: CLValue) {
    let dictionary_uref = get_dictionary(dictionary_name);
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_uref);
    let (dictionary_item_key_ptr, dictionary_item_key_size) = dictionary_item_key_to_ptr(key);

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let (cl_value_ptr, cl_value_size, _bytes) = to_ptr(value);

    let result = unsafe {
        let ret = ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            cl_value_ptr,
            cl_value_size
        );
        api_error::result_from(ret)
    };

    result.unwrap_or_revert()
}

/// Removes the [`Key`] stored under `dictionary_name` in the current context's named keys.
#[inline]
pub fn remove_dictionary(dictionary_name: &str) {
    runtime::remove_key(dictionary_name);
}

/// Gets a value under a key in a dictionary from the contract's storage.
pub fn get_dictionary_value(dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
    let dictionary_uref = get_dictionary(dictionary_name);
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_uref);
    let (dictionary_item_key_ptr, dictionary_item_key_size) = dictionary_item_key_to_ptr(key);

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_dictionary_get(
                uref_ptr,
                uref_size,
                dictionary_item_key_ptr,
                dictionary_item_key_size,
                value_size.as_mut_ptr()
            )
        };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    Some(Bytes::from(value_bytes))
}

fn get_dictionary(name: &str) -> URef {
    if name.is_empty() {
        runtime::revert(ApiError::MissingKey)
    }

    let dictionary_uref = match runtime::get_key(name) {
        Some(dictionary_key) => *dictionary_key.as_uref().unwrap_or_revert(),
        None => {
            let value_size = {
                let mut value_size = MaybeUninit::uninit();
                let ret = unsafe { ext_ffi::casper_new_dictionary(value_size.as_mut_ptr()) };
                api_error::result_from(ret).unwrap_or_revert();
                unsafe { value_size.assume_init() }
            };
            let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
            let uref: URef = bytesrepr::deserialize(value_bytes).unwrap_or_revert();
            runtime::put_key(name, Key::from(uref));
            uref
        }
    };
    dictionary_uref
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens(to: &Address, amount: &U512) {
    let main_purse = get_or_create_main_purse();

    match to {
        Address::Account(account) => {
            transfer_from_purse_to_account(main_purse, *account, *amount, None).unwrap_or_revert();
        }
        // todo: Why?
        Address::Contract(_) => revert(ExecutionError::TransferToContract.code())
    };
}

/// Writes an event to the contract's storage.
pub fn emit_event(event: &Bytes) {
    casper_event_standard::emit_bytes(event.clone())
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

/// Calls a contract method by Address
#[inline(always)]
pub fn call_contract(address: Address, call_def: CallDef) -> Bytes {
    let contract_package_hash = *address.as_contract_package_hash().unwrap_or_revert();
    let method = call_def.entry_point();
    let mut args = call_def.args().to_owned();
    if call_def.amount() == U512::zero() {
        call_versioned_contract(contract_package_hash, None, method, args)
    } else {
        let cargo_purse = get_or_create_cargo_purse();
        let main_purse = get_main_purse().unwrap_or_revert();

        transfer_from_purse_to_purse(main_purse, cargo_purse, call_def.amount(), None)
            .unwrap_or_revert_with(ApiError::Transfer);
        args.insert(consts::CARGO_PURSE_ARG, cargo_purse)
            .unwrap_or_revert();

        let result = call_versioned_contract(contract_package_hash, None, method, args);
        if !is_purse_empty(cargo_purse) {
            runtime::revert(ApiError::InvalidPurse)
        }
        result
    }
}

/// Gets the address of the currently run contract
#[inline(always)]
pub fn self_address() -> Address {
    let first_elem = take_call_stack_elem(0);
    call_stack_element_to_address(first_elem)
}

/// Gets the balance of the current contract.
#[inline(always)]
pub fn self_balance() -> U512 {
    let main_purse = get_or_create_main_purse();
    get_purse_balance(main_purse).unwrap_or_revert()
}

/// Invokes the specified `entry_point_name` of stored logic at a specific `contract_package_hash`
/// address, for the most current version of a contract package by default or a specific
/// `contract_version` if one is provided, and passing the provided `runtime_args` to it.
pub fn call_versioned_contract(
    contract_package_hash: ContractPackageHash,
    contract_version: Option<ContractVersion>,
    entry_point_name: &str,
    runtime_args: RuntimeArgs
) -> Bytes {
    let (contract_package_hash_ptr, contract_package_hash_size, _bytes) =
        to_ptr(contract_package_hash);
    let (contract_version_ptr, contract_version_size, _bytes) = to_ptr(contract_version);
    let (entry_point_name_ptr, entry_point_name_size, _bytes) = to_ptr(entry_point_name);
    let (runtime_args_ptr, runtime_args_size, _bytes) = to_ptr(runtime_args);

    let bytes_written = {
        let mut bytes_written = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_call_versioned_contract(
                contract_package_hash_ptr,
                contract_package_hash_size,
                contract_version_ptr,
                contract_version_size,
                entry_point_name_ptr,
                entry_point_name_size,
                runtime_args_ptr,
                runtime_args_size,
                bytes_written.as_mut_ptr()
            )
        };
        api_error::result_from(ret).unwrap_or_revert();
        unsafe { bytes_written.assume_init() }
    };
    odra_core::casper_types::bytesrepr::Bytes::from(deserialize_contract_result(bytes_written))
}

/// Reads from memory the amount attached to the current call.
pub fn attached_value() -> U512 {
    unsafe { ATTACHED_VALUE }
}

/// Stores in memory the amount attached to the current call.
pub fn set_attached_value(amount: U512) {
    unsafe {
        ATTACHED_VALUE = amount;
    }
}

/// Zeroes the amount attached to the current call.
pub fn clear_attached_value() {
    unsafe { ATTACHED_VALUE = U512::zero() }
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
    ret == 0
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
    let amount = get_purse_balance(cargo_purse);
    if let Some(amount) = amount {
        let contract_purse = get_or_create_main_purse();
        transfer_from_purse_to_purse(cargo_purse, contract_purse, amount, None).unwrap_or_revert();
        set_attached_value(amount);
    } else {
        revert(ExecutionError::NativeTransferError.code())
    }
}

/// Creates a new purse under the `__contract_main_purse` key for the currently executing contract
/// if it doesn't exist, or returns the existing main purse.
///
/// # Returns
///
/// The main purse as a [`URef`] if it already exists, otherwise a new purse is created and returned.
pub fn get_or_create_main_purse() -> URef {
    match get_main_purse() {
        Some(purse) => purse,
        None => {
            let purse = create_purse();
            runtime::put_key(consts::CONTRACT_MAIN_PURSE, purse.into());
            purse
        }
    }
}

/// Gets the main purse of the currently executing contract.
///
/// # Returns
///
/// The main purse as a [`URef`] if it exists, otherwise `None` is returned.
#[inline(always)]
pub fn get_main_purse() -> Option<URef> {
    runtime::get_key(consts::CONTRACT_MAIN_PURSE).and_then(|key| key.as_uref().cloned())
}

fn initial_named_keys(schemas: Schemas) -> NamedKeys {
    let mut named_keys = NamedKeys::new();
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

fn deserialize_contract_result(bytes_written: usize) -> Vec<u8> {
    if bytes_written == 0 {
        // If no bytes were written, the host buffer hasn't been set and hence shouldn't be read.
        vec![]
    } else {
        // NOTE: this is a copy of the contents of `read_host_buffer()`.  Calling that directly from
        // here causes several contracts to fail with a Wasmi `Unreachable` error.
        let bytes_non_null_ptr = contract_api::alloc_bytes(bytes_written);
        let mut dest: Vec<u8> = unsafe {
            Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), bytes_written, bytes_written)
        };
        read_host_buffer_into(&mut dest).unwrap_or_revert();
        dest
    }
}

fn take_call_stack_elem(n: usize) -> CallStackElement {
    runtime::get_call_stack()
        .into_iter()
        .nth_back(n)
        .unwrap_or_revert()
}

fn create_constructor_group(contract_package_hash: ContractPackageHash) -> URef {
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

fn revoke_access_to_constructor_group(
    contract_package_hash: ContractPackageHash,
    constructor_access: URef
) {
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(
        contract_package_hash,
        consts::CONSTRUCTOR_GROUP_NAME,
        urefs
    )
    .unwrap_or_revert();
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

fn is_purse_empty(purse: URef) -> bool {
    get_purse_balance(purse)
        .map(|balance| balance.is_zero())
        .unwrap_or_else(|| true)
}

fn get_or_create_cargo_purse() -> URef {
    match runtime::get_key(consts::CONTRACT_CARGO_PURSE) {
        Some(key) => *key.as_uref().unwrap_or_revert(),
        None => {
            let purse = create_purse();
            runtime::put_key(consts::CONTRACT_CARGO_PURSE, purse.into());
            purse
        }
    }
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

fn dictionary_item_key_to_ptr(dictionary_item_key: &[u8]) -> (*const u8, usize) {
    let ptr = dictionary_item_key.as_ptr();
    let size = dictionary_item_key.len();
    (ptr, size)
}

fn write(key: Key, value: CLValue) {
    let (key_ptr, key_size, _bytes) = to_ptr(key);
    let (value_ptr, value_size, _bytes) = to_ptr(value);

    unsafe {
        ext_ffi::casper_write(key_ptr, key_size, value_ptr, value_size);
    }
}

fn write_new(value: CLValue) -> URef {
    let uref_non_null_ptr = contract_api::alloc_bytes(UREF_SERIALIZED_LENGTH);
    let (cl_value_ptr, cl_value_size, _cl_value_bytes) = to_ptr(value);
    let bytes = unsafe {
        ext_ffi::casper_new_uref(uref_non_null_ptr.as_ptr(), cl_value_ptr, cl_value_size); // URef has `READ_ADD_WRITE`
        Vec::from_raw_parts(
            uref_non_null_ptr.as_ptr(),
            UREF_SERIALIZED_LENGTH,
            UREF_SERIALIZED_LENGTH
        )
    };
    deserialize(bytes).unwrap_or_revert()
}

fn read(key: Key) -> Option<Bytes> {
    let (key_ptr, key_size, _bytes) = to_ptr(key);
    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_read_value(key_ptr, key_size, value_size.as_mut_ptr()) };
        match api_error::result_from(ret) {
            Ok(_) => unsafe { value_size.assume_init() },
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    Some(Bytes::from(value_bytes))
}

fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

fn get_named_arg_size(name: &str) -> Result<usize, ApiError> {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize
        )
    };
    match ret {
        0 => Ok(arg_size),
        _ => Err(ApiError::from(ret as u32))
    }
}
