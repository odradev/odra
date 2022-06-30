use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    Address, CLTyped, CLValue, EventData, RuntimeArgs, CLType,
};

#[allow(improper_ctypes)]
extern "C" {
    fn __caller() -> Address;
    fn __set_var(key: &[u8], value: &CLValue);
    fn __get_var(key: &[u8]) -> Option<CLValue>;
    fn __set_bool(key: &[u8], value: bool);
    fn __get_bool(key: &[u8]) -> Option<bool>;
    fn __set_dict_value(dict: &[u8], key: &[u8], value: &CLValue);
    fn __get_dict_value(dict: &[u8], key: &[u8]) -> Option<CLValue>;
    fn __emit_event(event: &EventData);
    fn __call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        returned_type: &CLType,
    ) -> CLValue;
    fn __revert(reason: u32);
    fn __print(message: &str);
}

pub struct ContractEnv;

impl ContractEnv {
    pub fn caller() -> Address {
        unsafe { __caller() }
    }

    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        unsafe { __set_var(key.as_bytes(), &CLValue::from_t(value).unwrap()) }
    }

    pub fn get_var(key: &str) -> Option<CLValue> {
        unsafe { __get_var(key.as_bytes()) }
    }

    pub fn set_bool(key: &str, value: bool) {
        unsafe { __set_bool(key.as_bytes(), value) }
    }

    pub fn get_bool(key: &str) -> Option<bool> {
        unsafe { __get_bool(key.as_bytes()) }
    }

    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        unsafe {
            __set_dict_value(
                dict.as_bytes(),
                key.to_bytes().unwrap().as_slice(),
                &CLValue::from_t(value).unwrap(),
            )
        }
    }

    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        unsafe { __get_dict_value(dict.as_bytes(), key.to_bytes().unwrap().as_slice()) }
    }

    pub fn emit_event(event: &EventData) {
        unsafe { __emit_event(event) }
    }

    pub fn call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        returned_type: &CLType,
    ) -> CLValue {
        unsafe { __call_contract(address, entrypoint, args, returned_type) }
    }

    pub fn revert(reason: u32) {
        unsafe { __revert(reason) }
    }

    pub fn print(message: &str) {
        unsafe { __print(message) }
    }
}
