use crate::borrow_env;
use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    event::Event,
    Address, CLTyped, CLValue, ExecutionError, NativeToken, U256,
};

/// Exposes the public API to communicate with the host.
pub struct ContractEnv;

impl ContractEnv {
    /// Returns the address of currently executing contract.
    pub fn self_address() -> Address {
        borrow_env().callee()
    }

    /// Gets the address of the currently executing contract.
    pub fn caller() -> Address {
        borrow_env().caller()
    }

    /// Stores the `value` under `key`.
    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        borrow_env().set_var(key, &CLValue::from_t(value).unwrap())
    }

    /// Gets a value stored under `key`.
    pub fn get_var(key: &str) -> Option<CLValue> {
        borrow_env().get_var(key)
    }

    /// Puts a key-value into a collection.
    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        borrow_env().set_dict_value(
            dict,
            key.to_bytes().unwrap().as_slice(),
            &CLValue::from_t(value).unwrap(),
        )
    }

    /// Gets the value from the `dict` collection under `key`.
    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        borrow_env().get_dict_value(dict, key.to_bytes().unwrap().as_slice())
    }

    /// Sends an event to the execution environment.
    pub fn emit_event<T>(event: &T)
    where
        T: ToBytes + Event,
    {
        let event_data = event.to_bytes().unwrap();
        borrow_env().emit_event(&event_data);
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

    /// Returns the backend native token metadata
    pub fn native_token() -> NativeToken {
        NativeToken::new("Wrapped Plascoin", "WPLS", 10)
    }

    /// Deposits native token. The most likely to be used in the exchange for the minting of a wrapped native token.
    pub fn deposit_native_token(address: Address, amount: U256) {}

    /// Withdraws native token. The most likely to be used in the exchange for the burning of a wrapped native token.
    pub fn withdraw_native_token(recipient: Address, amount: U256) {}
}
