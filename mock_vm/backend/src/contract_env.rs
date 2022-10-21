//! Exposes the public API to communicate with the host.

use odra_mock_vm_types::{
    odra_types::{event::Event, ExecutionError},
    Address, Balance, BlockTime, OdraType, TypedValue,
};

use crate::borrow_env;

/// Returns the current block time.
pub fn get_block_time() -> BlockTime {
    borrow_env().get_block_time()
}

/// Gets the address of the currently executing contract.
pub fn caller() -> Address {
    borrow_env().caller()
}

/// Returns the address of currently executing contract.
pub fn self_address() -> Address {
    borrow_env().callee()
}

/// Stores the `value` under `key`.
pub fn set_var<T: OdraType>(key: &str, value: T) {
    borrow_env().set_var(key, TypedValue::from_t(value).unwrap())
}

/// Gets a value stored under `key`.
pub fn get_var<T: OdraType>(key: &str) -> Option<T> {
    borrow_env()
        .get_var(key)
        .map(|val| TypedValue::into_t(val).unwrap())
}

/// Puts a key-value into a collection.
pub fn set_dict_value<K: OdraType, V: OdraType>(dict: &str, key: &K, value: V) {
    borrow_env().set_dict_value(
        dict,
        key.to_bytes().unwrap().as_slice(),
        &TypedValue::from_t(value).unwrap(),
    )
}

/// Gets the value from the `dict` collection under `key`.
pub fn get_dict_value<K: OdraType, T: OdraType>(dict: &str, key: &K) -> Option<T> {
    let key = key.to_bytes().unwrap();
    let key = key.as_slice();
    borrow_env()
        .get_dict_value(dict, key)
        .map(|val| TypedValue::into_t(val).unwrap())
}

/// Stops execution of a contract and reverts execution effects with a given [`ExecutionError`].
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>,
{
    let execution_error: ExecutionError = error.into();
    borrow_env().revert(execution_error.into());
    panic!("OdraRevert")
}

/// Sends an event to the execution environment.
pub fn emit_event<T>(event: T)
where
    T: OdraType + Event,
{
    let event_data = event.to_bytes().unwrap();
    borrow_env().emit_event(&event_data);
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> Balance {
    borrow_env().attached_value()
}

/// Returns the value that represents one native token.
pub fn one_token() -> Balance {
    Balance::one()
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens(to: Address, amount: Balance) -> bool {
    let callee = borrow_env().callee();
    borrow_env().transfer_tokens(callee, to, amount)
}

/// Returns the balance of the account associated with the current contract.
pub fn self_balance() -> Balance {
    borrow_env().self_balance()
}
