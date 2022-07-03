use std::collections::HashMap;

use odra_types::{Address, CLTyped, CLValue, bytesrepr::{ToBytes, FromBytes, Bytes}, EventData, RuntimeArgs};
use ref_thread_local::RefThreadLocal;

mod context;
mod contract_container;
mod contract_register;
mod mock_vm;
mod storage;

pub use contract_container::EntrypointCall;

ref_thread_local::ref_thread_local!(
    static managed ENV: mock_vm::MockVm = mock_vm::MockVm::default();
);

pub fn borrow_env<'a>() -> ref_thread_local::Ref<'a, mock_vm::MockVm> {
    ENV.borrow()
}

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
}

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(
        name: &str,
        entrypoints: HashMap<String, EntrypointCall>,
        args: RuntimeArgs,
    ) -> Address {
        borrow_env().register_contract(name, entrypoints, args)
    }

    pub fn call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        _has_return: bool,
    ) -> Option<Bytes> {
        borrow_env().call_contract(address, entrypoint, args, _has_return)
    }

    pub fn backend_name() -> String {
        borrow_env().get_backend_name()
    }
}
