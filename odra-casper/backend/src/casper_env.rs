use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    DICTIONARY_ITEM_KEY_MAX_LENGTH
};
use lazy_static::lazy_static;
use odra_casper_types::{Address, OdraType};
use odra_types::event::OdraEvent;
use std::{collections::BTreeMap, sync::Mutex};

use casper_contract::{
    contract_api::{
        runtime,
        system::{create_purse, get_purse_balance, transfer_from_purse_to_purse}
    },
    unwrap_or_revert::UnwrapOrRevert
};

use casper_types::{
    system::CallStackElement, ApiError, CLTyped, ContractPackageHash, RuntimeArgs, URef, U512
};

use odra_casper_shared::consts;
use odra_types::ExecutionError;

lazy_static! {
    static ref SEEDS: Mutex<BTreeMap<String, URef>> = Mutex::new(BTreeMap::new());
    static ref KEYS: Mutex<BTreeMap<Vec<u8>, String>> = Mutex::new(BTreeMap::new());
}

/// Save value to the storage.
#[inline(always)]
pub fn set_key<T: OdraType>(name: &[u8], value: T) {
    // TODO: check key length
    save_value(name, value);
}

/// Read value from the storage.
#[inline(always)]
pub fn get_key<T: OdraType>(name: &[u8]) -> Option<T> {
    // TODO: check key length
    read_value(name)
}

#[inline(always)]
pub fn set_dict_value<K: OdraType, V: OdraType>(seed: &[u8], key: &K, value: V) {
    // TODO: optimize key creation

    let key = to_dictionary_key(seed, key);
    save_value(key.as_bytes(), value);
}

#[inline(always)]
pub fn get_dict_value<K: OdraType, V: OdraType>(seed: &[u8], key: &K) -> Option<V> {
    // TODO: optimize key creation

    let key = to_dictionary_key(seed, key);
    read_value(key.as_bytes())
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

/// Record event to the contract's storage.
#[inline(always)]
pub fn emit_event<T>(event: T)
where
    T: OdraType + OdraEvent
{
    casper_event_standard::emit(event);
}

/// Convert any key to hash.
fn _to_variable_key<T: ToBytes>(key: T) -> String {
    let preimage = key.to_bytes().unwrap_or_revert();
    let bytes = runtime::blake2b(preimage);
    hex::encode(bytes)
}

/// Convert any key to hash.
fn to_dictionary_key<T: ToBytes>(seed: &[u8], key: &T) -> String {
    match KEYS.lock() {
        Ok(mut keys) => {
            let key_bytes = key.to_bytes().unwrap_or_revert();

            let mut preimage = Vec::with_capacity(seed.len() + key_bytes.len());
            preimage.extend_from_slice(seed);
            preimage.extend_from_slice(&key_bytes);

            match keys.get(&preimage) {
                Some(key) => key.to_owned(),
                None => {
                    let bytes = runtime::blake2b(&preimage);
                    let key = hex::encode(bytes);
                    keys.insert(preimage, key.clone());
                    key
                }
            }
        }
        Err(_) => runtime::revert(ApiError::ValueNotFound)
    }
}

/// Calls a contract method by Address
#[inline(always)]
pub fn call_contract<T: CLTyped + FromBytes>(
    contract_package_hash: ContractPackageHash,
    entry_point: &str,
    args: RuntimeArgs
) -> T {
    runtime::call_versioned_contract(contract_package_hash, None, entry_point, args)
}

#[inline(always)]
pub fn call_contract_with_amount<T: CLTyped + FromBytes>(
    contract_package_hash: ContractPackageHash,
    entry_point: &str,
    args: RuntimeArgs,
    amount: U512
) -> T {
    let cargo_purse = create_purse();
    let main_purse = get_or_create_purse();

    let mut args = args;
    transfer_from_purse_to_purse(main_purse, cargo_purse, amount, None)
        .unwrap_or_revert_with(ApiError::Transfer);
    args.insert(consts::CARGO_PURSE_ARG, cargo_purse)
        .unwrap_or_revert();
    let result = call_contract(contract_package_hash, entry_point, args);

    if !is_purse_empty(cargo_purse) {
        runtime::revert(ApiError::InvalidPurse)
    }

    result
}

#[inline(always)]
pub fn get_block_time() -> u64 {
    runtime::get_blocktime().into()
}

#[inline(always)]
pub fn revert(error: u16) -> ! {
    runtime::revert(ApiError::User(error))
}

pub fn get_or_create_purse() -> URef {
    match runtime::get_key(consts::CONTRACT_MAIN_PURSE) {
        Some(purse_key) => *purse_key.as_uref().unwrap_or_revert(),
        None => {
            let purse = create_purse();
            runtime::put_key(consts::CONTRACT_MAIN_PURSE, purse.into());
            purse
        }
    }
}

#[inline(always)]
pub fn self_balance() -> U512 {
    let purse = get_or_create_purse();
    get_purse_balance(purse).unwrap_or_default()
}

fn is_purse_empty(purse: URef) -> bool {
    get_purse_balance(purse)
        .map(|balance| balance.is_zero())
        .unwrap_or_else(|| true)
}

fn get_state_uref() -> URef {
    get_seed(consts::STATE_KEY, || {
        let key = runtime::get_key(consts::STATE_KEY).unwrap_or_revert();
        let state_uref: URef = *key.as_uref().unwrap_or_revert();
        state_uref
    })
}

fn get_seed<F>(name: &str, initializer: F) -> URef
where
    F: FnOnce() -> URef
{
    match SEEDS.lock() {
        Ok(mut seeds) => match seeds.get(name) {
            Some(seed) => *seed,
            None => {
                let seed = initializer();
                seeds.insert(String::from(name), seed);
                seed
            }
        },
        Err(_) => runtime::revert(ApiError::ValueNotFound)
    }
}

#[inline(always)]
pub fn save_value<T: OdraType>(key: &[u8], value: T) {
    let state_uref = get_state_uref();
    dictionary_put(state_uref, key, value);
}

#[inline(always)]
pub fn read_value<T: OdraType>(key: &[u8]) -> Option<T> {
    let state_uref = get_state_uref();
    dictionary_get(state_uref, key).unwrap_or_revert()
}

pub fn dictionary_put<V: CLTyped + ToBytes>(
    dictionary_seed_uref: URef,
    dictionary_item_key: &[u8],
    value: V
) {
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_seed_uref);

    let dictionary_item_key_ptr = dictionary_item_key.as_ptr();
    let dictionary_item_key_size = dictionary_item_key.len();

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let cl_value = casper_types::CLValue::from_t(value).unwrap_or_revert();
    let (cl_value_ptr, cl_value_size, _bytes) = to_ptr(cl_value);

    let result = unsafe {
        let ret = casper_contract::ext_ffi::casper_dictionary_put(
            uref_ptr,
            uref_size,
            dictionary_item_key_ptr,
            dictionary_item_key_size,
            cl_value_ptr,
            cl_value_size
        );
        casper_types::api_error::result_from(ret)
    };

    result.unwrap_or_revert()
}

fn dictionary_get<V: CLTyped + FromBytes>(
    dictionary_seed_uref: URef,
    dictionary_item_key: &[u8]
) -> Result<Option<V>, casper_types::bytesrepr::Error> {
    let (uref_ptr, uref_size, _bytes1) = to_ptr(dictionary_seed_uref);
    let dictionary_item_key_ptr = dictionary_item_key.as_ptr();
    let dictionary_item_key_size = dictionary_item_key.len();

    if dictionary_item_key_size > DICTIONARY_ITEM_KEY_MAX_LENGTH {
        runtime::revert(ApiError::DictionaryItemKeyExceedsLength)
    }

    let value_size = {
        let mut value_size = std::mem::MaybeUninit::uninit();
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
            Err(ApiError::ValueNotFound) => return Ok(None),
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    Ok(Some(casper_types::bytesrepr::deserialize(value_bytes)?))
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
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
    let mut bytes_written = std::mem::MaybeUninit::uninit();
    let ret = unsafe {
        casper_contract::ext_ffi::casper_read_host_buffer(
            dest.as_mut_ptr(),
            dest.len(),
            bytes_written.as_mut_ptr()
        )
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    casper_types::api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}
