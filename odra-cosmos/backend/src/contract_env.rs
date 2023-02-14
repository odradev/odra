//! Casper backend for WASM.
//!
//! It provides all the required functions to communicate between Odra and Casper.

use cosmwasm_std::{Response, Binary};
use odra_cosmos_shared::{native_token::NativeTokenMetadata, utils::build_wasm_message};
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
#[cfg(target_arch = "wasm32")]
pub fn set_dict_value<K: OdraType, V: OdraType>(dict: &str, key: &K, value: V) {
    use cosmwasm_std::Storage;

    let mut storage = cosmwasm_std::ExternalStorage::new();

    let key = key.ser().unwrap();
    let key = dict
        .as_bytes()
        .iter()
        .chain(&key)
        .map(|v| v.to_owned())
        .collect::<Vec<_>>();
    storage.set(&key, value.ser().unwrap().as_slice());
}

/// Read value from the mapping.
#[cfg(target_arch = "wasm32")]
pub fn get_dict_value<K: OdraType, T: OdraType>(_dict: &str, _key: &K) -> Option<T> {
    unimplemented!()
}

/// Revert the execution.
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>
{
    let error: ExecutionError = error.into();
    panic!("{}", error.message());
}

/// Emits event.
pub fn emit_event<T>(event: T)
where
    T: OdraEvent + Into<cosmwasm_std::Event>
{
    RT.with(|runtime| runtime.borrow_mut().add_event(event.into()))
}

/// Call another contract.
pub fn call_contract<T: OdraType>(
    address: Address,
    entrypoint: &str,
    args: CallArgs,
    _amount: Option<Balance>
) -> T {
    RT.with(|runtime| {
        let cosmos_address: cosmwasm_std::Addr = address.into();
        let msg = build_wasm_message(entrypoint, args);
        let call =  cosmwasm_std::WasmMsg::Execute {
            contract_addr: cosmos_address.into_string(),
            msg: Binary(msg),
            funds: vec![],
        };
        runtime.borrow_mut().add_call(call);
        if let Ok(null) = T::deser(vec![110, 117, 108, 108]) {
            null
        } else {
            unimplemented!()
        }
    })
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
pub fn transfer_tokens<B: Into<Balance>>(_to: Address, _amount: B) {
    unimplemented!()
}

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}

pub fn get_response() -> Response {
    RT.with(|runtime| runtime.borrow().get_response())
}
