use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use odra_test_env::ContractContainer;
use odra_types::{Address, RuntimeArgs, bytesrepr::Bytes};

#[derive(WrapperApi)]
pub struct TestBackend {
    backend_name: fn() -> String,
    register_contract: fn(container: &ContractContainer) -> Address,
    call_contract: fn(addr: &Address, entrypoint: &str, args: &RuntimeArgs) -> Bytes,
}

pub struct TestEnvWrapper {
    test_backend: Container<TestBackend>,
}

impl Default for TestEnvWrapper {
    fn default() -> Self {
        TestEnvWrapper {
            test_backend: unsafe {
                Container::load("libodra_test_env.so")
                    .expect("Could not open library or load symbols")
            },
        }
    }
}

impl TestEnvWrapper {
    pub fn backend_name(&self) -> String {
        self.test_backend.backend_name()
    }

    pub fn register_contract(&self, container: &ContractContainer) -> Address {
        self.test_backend.register_contract(container)
    }

    pub fn call_contract(&self, addr: &Address, entrypoint: &str, args: &RuntimeArgs) -> Bytes {
        self.test_backend.call_contract(addr, entrypoint, args)
    }
}
