use casper_types::{
    api_error,
    bytesrepr::{self, FromBytes, ToBytes},
    EntryPoints
};
use lazy_static::lazy_static;
use odra_casper_types::{Address, OdraType};
use odra_types::event::OdraEvent;
use std::{collections::BTreeMap, mem::MaybeUninit, sync::Mutex};

use casper_contract::{
    contract_api::{
        self, runtime,
        storage::{self, dictionary_put},
        system::{create_purse, get_purse_balance, transfer_from_purse_to_purse}
    },
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert
};

use casper_types::{
    system::CallStackElement, ApiError, CLTyped, ContractPackageHash, Key, RuntimeArgs, URef, U512
};

use odra_casper_shared::consts;
use odra_types::ExecutionError;

lazy_static! {
    static ref SEEDS: Mutex<BTreeMap<String, URef>> = Mutex::new(BTreeMap::new());
}

const STATE_KEY: &str = "state";

pub fn add_contract_version(contract_package_hash: ContractPackageHash, entry_points: EntryPoints) {
    let mut named_keys = casper_types::contracts::NamedKeys::new();
    named_keys.insert(String::from(STATE_KEY), Key::URef(get_new_dict_uref()));
    casper_contract::contract_api::storage::add_contract_version(
        contract_package_hash,
        entry_points,
        named_keys
    );
    runtime::remove_key(STATE_KEY);
}

/// Save value to the storage.
pub fn set_key<T: OdraType>(name: &str, value: T) {
    save_value(&to_variable_key(name), value);
}

/// Read value from the storage.
pub fn get_key<T: OdraType>(name: &str) -> Option<T> {
    read_value(&to_variable_key(name))
}

pub fn set_dict_value<K: OdraType, V: OdraType>(seed: &str, key: &K, value: V) {
    save_value(&to_dictionary_key(seed, key), value);
}

pub fn get_dict_value<K: OdraType, V: OdraType>(seed: &str, key: &K) -> Option<V> {
    read_value(&to_dictionary_key(seed, key))
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
    T: OdraType + OdraEvent
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
fn to_variable_key<T: ToBytes>(key: T) -> String {
    let preimage = key.to_bytes().unwrap_or_revert();
    let bytes = runtime::blake2b(preimage);
    hex::encode(bytes)
}

/// Convert any key to hash.
fn to_dictionary_key<T: ToBytes>(seed: &str, key: &T) -> String {
    let seed_bytes = seed.as_bytes();
    let key_bytes = key.to_bytes().unwrap_or_revert();

    let mut preimage = Vec::with_capacity(seed_bytes.len() + key_bytes.len());
    preimage.extend_from_slice(seed_bytes);
    preimage.extend_from_slice(&key_bytes);
    let bytes = runtime::blake2b(preimage);
    hex::encode(bytes)
}

/// Calls a contract method by Address
pub fn call_contract<T: CLTyped + FromBytes>(
    contract_package_hash: ContractPackageHash,
    entry_point: &str,
    args: RuntimeArgs
) -> T {
    runtime::call_versioned_contract(contract_package_hash, None, entry_point, args)
}

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

pub fn get_block_time() -> u64 {
    runtime::get_blocktime().into()
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

fn get_new_dict_uref() -> URef {
    let value_size = {
        let mut value_size = MaybeUninit::uninit();
        let ret = unsafe { ext_ffi::casper_new_dictionary(value_size.as_mut_ptr()) };
        api_error::result_from(ret).unwrap_or_revert();
        unsafe { value_size.assume_init() }
    };

    let mut dest: Vec<u8> = {
        let bytes_non_null_ptr = contract_api::alloc_bytes(value_size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), value_size, value_size) }
    };

    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    api_error::result_from(ret).unwrap_or_revert();
    unsafe { bytes_written.assume_init() };

    bytesrepr::deserialize(dest).unwrap_or_revert()
}

fn get_state_uref() -> URef {
    let key = runtime::get_key(STATE_KEY).unwrap_or_revert();
    key.try_into().unwrap_or_revert()
}

pub fn save_value<T: OdraType>(key: &str, value: T) {
    let state_uref = get_state_uref();
    storage::dictionary_put(state_uref, key, value);
}

pub fn read_value<T: OdraType>(key: &str) -> Option<T> {
    let state_uref = get_state_uref();
    storage::dictionary_get(state_uref, key).unwrap_or_revert()
}
