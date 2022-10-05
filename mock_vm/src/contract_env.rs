use crate::borrow_env;
use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    event::Event,
    Address, CLTyped, CLValue, ExecutionError, U512,
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

    /// Returns the current block time.
    pub fn get_block_time() -> u64 {
        borrow_env().get_block_time()
    }

    /// Returns amount of native token attached to the call.
    pub fn attached_value() -> U512 {
        borrow_env().attached_value()
    }

    /// Returns the value that represents one native token.
    pub fn one_token() -> U512 {
        U512::one()
    }

    /// Attaches [amount] of native token to the next contract call.
    pub fn with_tokens(amount: U512) {
        borrow_env().attach_value(amount);
    }

    /// Returns the balance of the account associated with the given address.
    pub fn token_balance(address: Address) -> U512 {
        borrow_env().token_balance(address)
    }

    /// Transfers native token from the contract caller to the given address.
    pub fn transfer_tokens(to: Address, amount: U512) -> bool {
        borrow_env().transfer_tokens(&to, amount)
    }

    /// Returns the balance of the account associated with the current contract.
    pub fn self_balance() -> U512 {
        borrow_env().self_balance()
    }
}
