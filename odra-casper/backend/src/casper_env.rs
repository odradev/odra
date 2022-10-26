use lazy_static::lazy_static;
use odra_casper_types::{odra_types::event::Event, Address, OdraType};
use std::{collections::BTreeMap, sync::Mutex};

use casper_contract::{
    contract_api::{
        self, runtime,
        storage::{self, dictionary_put},
        system::{create_purse, get_purse_balance, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error, bytesrepr::ToBytes, system::CallStackElement, ApiError, ContractPackageHash,
    ContractVersion, Key, RuntimeArgs, URef, U512,
};

use odra_casper_shared::consts;

lazy_static! {
    static ref SEEDS: Mutex<BTreeMap<String, URef>> = Mutex::new(BTreeMap::new());
}

/// Save value to the storage.
pub fn set_key<T: OdraType>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

/// Read value from the storage.
pub fn get_key<T: OdraType>(name: &str) -> Option<T> {
    match runtime::get_key(name) {
        None => None,
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            let value = storage::read(key).unwrap_or_revert().unwrap_or_revert();
            Some(value)
        }
    }
}

pub fn set_dict_value<K: OdraType, V: OdraType>(seed: &str, key: &K, value: V) {
    let seed = get_seed(seed);
    storage::dictionary_put(seed, &to_dictionary_key(key), value);
}

pub fn get_dict_value<K: OdraType, V: OdraType>(seed: &str, key: &K) -> Option<V> {
    let seed = get_seed(seed);
    storage::dictionary_get(seed, &to_dictionary_key(key)).unwrap_or_revert()
}

/// Returns address based on a [`CallStackElement`].
///
/// For `Session` and `StoredSession` variants it will return account hash, and for `StoredContract`
/// case it will use contract hash as the address.
fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Address {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Address::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session
            // wants to interact, caller's address will be used.
            Address::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Address::from(contract_package_hash),
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
pub fn caller() -> Address {
    let second_elem = take_call_stack_elem(1);
    call_stack_element_to_address(second_elem)
}

/// Gets the address of the currently run contract
pub fn self_address() -> Address {
    let first_elem = take_call_stack_elem(0);
    call_stack_element_to_address(first_elem)
}

/// Record event to the contract's storage.
pub fn emit_event<T>(event: T)
where
    T: OdraType + Event,
{
    let (events_length, key): (u32, URef) = match runtime::get_key(consts::EVENTS_LENGTH) {
        None => {
            let key = storage::new_uref(0u32);
            runtime::put_key(consts::EVENTS_LENGTH, Key::from(key));
            (0u32, key)
        }
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            let value = storage::read(key).unwrap_or_revert().unwrap_or_revert();
            (value, key)
        }
    };
    let events_seed: URef = get_seed(consts::EVENTS);
    dictionary_put(events_seed, &events_length.to_string(), event);
    storage::write(key, events_length + 1);
}

/// Convert any key to hash.
pub fn to_dictionary_key<T: OdraType>(key: &T) -> String {
    let preimage = key.to_bytes().unwrap_or_revert();
    let bytes = runtime::blake2b(preimage);
    hex::encode(bytes)
}

/// Calls a contract method by Address
pub fn call_contract(
    contract_package_hash: ContractPackageHash,
    entry_point: &str,
    runtime_args: RuntimeArgs,
) -> Vec<u8> {
    let contract_version: Option<ContractVersion> = None;

    let (contract_package_hash_ptr, contract_package_hash_size, _bytes) =
        to_ptr(contract_package_hash);
    let (contract_version_ptr, contract_version_size, _bytes) = to_ptr(contract_version);
    let (entry_point_name_ptr, entry_point_name_size, _bytes) = to_ptr(entry_point);
    let (runtime_args_ptr, runtime_args_size, _bytes) = to_ptr(runtime_args);

    let bytes_written = {
        let mut bytes_written = std::mem::MaybeUninit::uninit();
        let ret = unsafe {
            casper_contract::ext_ffi::casper_call_versioned_contract(
                contract_package_hash_ptr,
                contract_package_hash_size,
                contract_version_ptr,
                contract_version_size,
                entry_point_name_ptr,
                entry_point_name_size,
                runtime_args_ptr,
                runtime_args_size,
                bytes_written.as_mut_ptr(),
            )
        };
        api_error::result_from(ret).unwrap_or_revert();
        unsafe { bytes_written.assume_init() }
    };

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

pub fn call_contract_with_amount(
    contract_package_hash: ContractPackageHash,
    entry_point: &str,
    runtime_args: RuntimeArgs,
    amount: U512,
) -> Vec<u8> {
    let cargo_purse = create_purse();
    let main_purse = get_or_create_purse();

    let mut args = runtime_args;
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

pub fn get_block_time() -> u64 {
    u64::from(runtime::get_blocktime())
}

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

pub fn self_balance() -> U512 {
    let purse = get_or_create_purse();
    get_purse_balance(purse).unwrap_or_default()
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = std::mem::MaybeUninit::uninit();
    let ret = unsafe {
        casper_contract::ext_ffi::casper_read_host_buffer(
            dest.as_mut_ptr(),
            dest.len(),
            bytes_written.as_mut_ptr(),
        )
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

fn get_seed(name: &str) -> URef {
    let mut seeds = SEEDS.lock().unwrap();
    let maybe_seed = seeds.get(name);
    match maybe_seed {
        Some(seed) => *seed,
        None => {
            let key: Key = match runtime::get_key(name) {
                Some(key) => key,
                None => {
                    storage::new_dictionary(name).unwrap_or_revert();
                    runtime::get_key(name).unwrap_or_revert()
                }
            };
            let seed: URef = *key.as_uref().unwrap_or_revert();
            seeds.insert(String::from(name), seed);
            seed
        }
    }
}

fn is_purse_empty(purse: URef) -> bool {
    get_purse_balance(purse)
        .map(|balance| balance.is_zero())
        .unwrap_or_else(|| true)
}
