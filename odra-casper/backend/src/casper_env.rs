use casper_types::bytesrepr::{FromBytes, ToBytes};
use odra_casper_types::{Address, Key, OdraType};
use odra_types::event::OdraEvent;

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

use odra_casper_shared::{
    consts,
    key_maker::{KeyMaker, StorageKey}
};
use odra_types::ExecutionError;

lazy_static::lazy_static! {
    static ref STATE: URef = {
        let key = runtime::get_key(consts::STATE_KEY).unwrap_or_revert();
        let state_uref: URef = *key.as_uref().unwrap_or_revert();
        state_uref
    };

    static ref STATE_BYTES: std::vec::Vec<u8> = {
        (*STATE).into_bytes().unwrap_or_revert()
    };
}

struct CasperKeyMaker;

impl KeyMaker for CasperKeyMaker {
    const DICTIONARY_ITEM_KEY_MAX_LENGTH: usize = casper_types::DICTIONARY_ITEM_KEY_MAX_LENGTH;

    fn blake2b(preimage: &[u8]) -> [u8; 32] {
        runtime::blake2b(preimage)
    }
}

/// Save value to the storage.
#[inline(always)]
pub fn set_key<T: OdraType>(name: &[u8], value: T) {
    save_value(CasperKeyMaker::to_variable_key(name), value)
}

/// Read value from the storage.
#[inline(always)]
pub fn get_key<T: OdraType>(name: &[u8]) -> Option<T> {
    read_value(CasperKeyMaker::to_variable_key(name))
}

#[inline(always)]
pub fn set_dict_value<K: OdraType + Key, V: OdraType>(seed: &[u8], key: &K, value: V) {
    save_value(CasperKeyMaker::to_dictionary_key(seed, key), value)
}

#[inline(always)]
pub fn get_dict_value<K: OdraType + Key, V: OdraType>(seed: &[u8], key: &K) -> Option<V> {
    read_value(CasperKeyMaker::to_dictionary_key(seed, key))
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

pub fn save_value<T: OdraType>(key: StorageKey, value: T) {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let (dictionary_item_key_ptr, dictionary_item_key_size) = key.to_ptr();


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

pub fn read_value<T: OdraType>(key: StorageKey) -> Option<T> {
    let uref_ptr = (*STATE_BYTES).as_ptr();
    let uref_size = (*STATE_BYTES).len();

    let (dictionary_item_key_ptr, dictionary_item_key_size) = key.to_ptr();

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
            Err(ApiError::ValueNotFound) => return None,
            Err(e) => runtime::revert(e)
        }
    };

    let value_bytes = read_host_buffer(value_size).unwrap_or_revert();
    
    let res = match casper_types::bytesrepr::deserialize(value_bytes) {
        Ok(res) => Ok(Some(res)),
        Err(e) => Err(e),
    };
    res.unwrap_or_revert()
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
