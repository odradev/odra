//! Casper backend for WASM.
//!
//! It provides all the required functions to communicate between Odra and Casper.

use odra_cosmos_shared::native_token::NativeTokenMetadata;
use odra_cosmos_types::{Address, Balance, BlockTime, CallArgs, OdraType};
use odra_types::{event::OdraEvent, ExecutionError};

use crate::runtime::RT;

/// Returns blocktime.
pub fn get_block_time() -> BlockTime {
    RT.with(|runtime| runtime.borrow().block_time())
}

/// Returns contract caller.
pub fn caller() -> Address {
    RT.with(|runtime| runtime.borrow().caller())
}

/// Returns current contract address.
pub fn self_address() -> Address {
    RT.with(|runtime| runtime.borrow().contract_address())
}

/// Store a value into the storage.
#[cfg(target_arch = "wasm32")]
pub fn set_var<T: OdraType>(key: &str, value: T) {
    use cosmwasm_std::Storage;

    let mut storage = cosmwasm_std::ExternalStorage::new();
    storage.set(key.as_bytes(), value.ser().unwrap().as_slice());
}

/// Read value from the storage.
#[cfg(target_arch = "wasm32")]
pub fn get_var<T: OdraType>(key: &str) -> Option<T> {
    use cosmwasm_std::Storage;

    let storage = cosmwasm_std::ExternalStorage::new();
    let bytes = storage.get(key.as_bytes());
    bytes.and_then(|b| T::deser(b).ok())
}

/// Store the mapping value under a given key.
pub fn set_dict_value<K: OdraType, V: OdraType>(dict: &str, key: &K, value: V) {
    unimplemented!()
}

/// Read value from the mapping.
pub fn get_dict_value<K: OdraType, T: OdraType>(dict: &str, key: &K) -> Option<T> {
    unimplemented!()
}

/// Revert the execution.
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>
{
    unimplemented!()
}

/// Emits event.
pub fn emit_event<T>(event: T)
where
    T: OdraType + OdraEvent
{
    unimplemented!()
}

/// Call another contract.
pub fn call_contract<T: OdraType>(
    address: Address,
    entrypoint: &str,
    args: CallArgs,
    amount: Option<Balance>
) -> T {
    unimplemented!()
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> Balance {
    unimplemented!()
}

/// Returns the balance of the account associated with the currently executing contract.
pub fn self_balance() -> Balance {
    unimplemented!()
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> Balance {
    unimplemented!()
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens<B: Into<Balance>>(to: Address, amount: B) {
    unimplemented!()
}

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}
