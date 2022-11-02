//! Exposes the public API to communicate with the host.

use odra_mock_vm_types::{Address, Balance, BlockTime, MockVMType, OdraType};
use odra_types::ExecutionError;

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
pub fn set_var<T: MockVMType>(key: &str, value: T) {
    borrow_env().set_var(key, value)
}

/// Gets a value stored under `key`.
pub fn get_var<T: OdraType>(key: &str) -> Option<T> {
    borrow_env().get_var(key)
}

/// Puts a key-value into a collection.
pub fn set_dict_value<K: MockVMType, V: MockVMType>(dict: &str, key: &K, value: V) {
    borrow_env().set_dict_value(dict, key.ser().unwrap().as_slice(), value)
}

/// Gets the value from the `dict` collection under `key`.
pub fn get_dict_value<K: MockVMType, T: MockVMType>(dict: &str, key: &K) -> Option<T> {
    let key = key.ser().unwrap();
    let key = key.as_slice();
    borrow_env().get_dict_value(dict, key)
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
pub fn emit_event<T: MockVMType>(event: T) {
    let event_data = event.ser().unwrap();
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
