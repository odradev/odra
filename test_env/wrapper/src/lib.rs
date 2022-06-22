use std::cell::RefCell;

use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use odra_test_env::ContractContainer;
use odra_types::{bytesrepr::Bytes, Address, RuntimeArgs};

thread_local! {
    static TEST_ENV: RefCell<Container<TestBackend>> = RefCell::new(unsafe {
        Container::load("libodra_test_env.so")
            .expect("Could not open library or load symbols")
    });
}


#[derive(WrapperApi)]
pub struct TestBackend {
    backend_name: fn() -> String,
    register_contract: fn(container: &ContractContainer) -> Address,
    call_contract: fn(addr: &Address, entrypoint: &str, args: &RuntimeArgs, has_return: bool) -> Bytes,
}

pub struct TestEnvWrapper {
    test_backend: Container<TestBackend>,
}

impl TestEnvWrapper {
    pub fn backend_name(&self) -> String {
        self.test_backend.backend_name()
    }

    pub fn register_contract(&self, container: &ContractContainer) -> Address {
        self.test_backend.register_contract(container)
    }

    pub fn call_contract(&self, addr: &Address, entrypoint: &str, args: &RuntimeArgs, has_return: bool) -> Bytes {
        self.test_backend.call_contract(addr, entrypoint, args, has_return)
    }
}

pub fn on_backend<F, R>(f: F) -> R 
where F: FnOnce(&Container<TestBackend>) -> R {
    TEST_ENV.with(|env| {
        let a = &*env.borrow();
        f(a)
    })
}
