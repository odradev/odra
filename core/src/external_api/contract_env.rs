use odra_types::{
    event::Event,
    bytesrepr::{FromBytes, ToBytes},
    Address, CLTyped, CLValue, EventData, RuntimeArgs, OdraError
};
use crate::UnwrapOrRevert;

#[allow(improper_ctypes)]
extern "C" {
    fn __caller() -> Address;
    fn __set_var(key: &[u8], value: &CLValue);
    fn __get_var(key: &[u8]) -> Option<CLValue>;
    fn __set_dict_value(dict: &[u8], key: &[u8], value: &CLValue);
    fn __get_dict_value(dict: &[u8], key: &[u8]) -> Option<CLValue>;
    fn __emit_event(event: &EventData);
    fn __call_contract(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> Vec<u8>;
    fn __revert(reason: &OdraError) -> !;
    fn __print(message: &str);
}

pub struct ContractEnv;

impl ContractEnv {
    pub fn caller() -> Address {
        unsafe { __caller() }
    }

    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        unsafe { 
            let cl_value = CLValue::from_t(value).unwrap_or_revert();
            __set_var(key.as_bytes(), &cl_value) 
        }
    }

    pub fn get_var(key: &str) -> Option<CLValue> {
        unsafe { __get_var(key.as_bytes()) }
    }

    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        unsafe {
            __set_dict_value(
                dict.as_bytes(),
                key.to_bytes().unwrap_or_revert().as_slice(),
                &CLValue::from_t(value).unwrap_or_revert(),
            )
        }
    }

    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        unsafe { 
            __get_dict_value(
                dict.as_bytes(), 
                key.to_bytes().unwrap_or_revert().as_slice()
            )
        }
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
        E: Into<OdraError>,
    {
        unsafe { __revert(&error.into()) }
    }


    pub fn print(message: &str) {
        unsafe { __print(message) }
    }
}
