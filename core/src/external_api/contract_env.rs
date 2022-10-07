use crate::UnwrapOrRevert;
use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    event::Event,
    Address, CLTyped, CLValue, EventData, ExecutionError, RuntimeArgs, U512,
};

#[allow(improper_ctypes)]
extern "Rust" {
    fn __get_block_time() -> u64;
    fn __self_address() -> Address;
    fn __caller() -> Address;
    fn __set_var(key: &str, value: &CLValue);
    fn __get_var(key: &str) -> Option<CLValue>;
    fn __set_dict_value(dict: &str, key: &[u8], value: &CLValue);
    fn __get_dict_value(dict: &str, key: &[u8]) -> Option<CLValue>;
    fn __emit_event(event: &EventData);
    fn __call_contract(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> Vec<u8>;
    fn __revert(reason: &ExecutionError) -> !;
    fn __print(message: &str);
    fn __self_balance() -> U512;
    fn __one_token() -> U512;
    fn __attached_value() -> U512;
    fn __with_tokens(amount: U512);
    fn __transfer_tokens(to: Address, amount: U512);
}

pub struct ContractEnv;

impl ContractEnv {
    pub fn get_block_time() -> u64 {
        unsafe { __get_block_time() }
    }

    pub fn self_address() -> Address {
        unsafe { __self_address() }
    }

    pub fn caller() -> Address {
        unsafe { __caller() }
    }

    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        unsafe {
            let cl_value = CLValue::from_t(value).unwrap_or_revert();
            __set_var(key, &cl_value)
        }
    }

    pub fn get_var(key: &str) -> Option<CLValue> {
        unsafe { __get_var(key) }
    }

    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        unsafe {
            __set_dict_value(
                dict,
                key.to_bytes().unwrap_or_revert().as_slice(),
                &CLValue::from_t(value).unwrap_or_revert(),
            )
        }
    }

    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        unsafe { __get_dict_value(dict, key.to_bytes().unwrap_or_revert().as_slice()) }
    }

    pub fn emit_event<T>(event: &T)
    where
        T: ToBytes + Event,
    {
        let event_data = event.to_bytes().unwrap_or_revert();
        unsafe { __emit_event(&event_data) }
    }

    pub fn call_contract(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> Vec<u8> {
        unsafe { __call_contract(address, entrypoint, args) }
    }

    pub fn revert<E>(error: E) -> !
    where
        E: Into<ExecutionError>,
    {
        unsafe { __revert(&error.into()) }
    }

    pub fn print(message: &str) {
        unsafe { __print(message) }
    }

    pub fn attached_value() -> U512 {
        unsafe { __attached_value() }
    }

    pub fn self_balance() -> U512 {
        unsafe { __self_balance() }
    }

    pub fn one_token() -> U512 {
        unsafe { __one_token() }
    }

    pub fn with_tokens(amount: U512) {
        unsafe { __with_tokens(amount) }
    }

    pub fn transfer_tokens(to: Address, amount: U512) {
        unsafe { __transfer_tokens(to, amount) }
    }
}
