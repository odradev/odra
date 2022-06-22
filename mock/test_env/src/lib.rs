mod events;
mod internal;

use std::cell::RefCell;

use odra_test_env::{ContractCollection, ContractContainer};
use odra_types::{bytesrepr::Bytes, Address, RuntimeArgs};

thread_local! {
    static STORAGE: RefCell<ContractCollection> = RefCell::new(ContractCollection::default());
}

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(container: &ContractContainer) -> Address {
        STORAGE.with(|storage| {
            let address = crate::internal::next_address();
            let mut storage = storage.borrow_mut();
            storage.add(address.clone(), container.clone());
            address
        })
    }

    pub fn call_contract(address: &Address, entrypoint: &str, args: &RuntimeArgs, _has_return: bool) -> Option<Bytes> {
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            storage.call(address, String::from(entrypoint), args.clone())
        })
    }

    pub fn backend_name() -> String {
        "Mock".to_string()
    }
}
