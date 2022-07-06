use odra_types::{Address, CLTyped, CLValue, bytesrepr::{ToBytes, FromBytes}, EventData, OdraError};

use crate::borrow_env;
pub struct ContractEnv;

impl ContractEnv {
    pub fn caller() -> Address {
        borrow_env().caller()
    }

    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        borrow_env().set_var(key.as_bytes(), &CLValue::from_t(value).unwrap())
    }

    pub fn get_var(key: &str) -> Option<CLValue> {
        borrow_env().get_var(key.as_bytes())
    }

    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        borrow_env().set_dict_value(
            dict.as_bytes(),
            key.to_bytes().unwrap().as_slice(),
            &CLValue::from_t(value).unwrap(),
        )
    }

    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        borrow_env().get_dict_value(dict.as_bytes(), key.to_bytes().unwrap().as_slice())
    }

    pub fn emit_event(event: &EventData) {
        todo!()
    }

    pub fn revert(error: OdraError) {
        borrow_env().revert(error)
    }
}
