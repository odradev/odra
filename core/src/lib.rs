pub mod contract_def;
mod event;
pub mod instance;
mod mapping;
mod variable;

pub use odra_proc_macros::{contract, instance, external_contract, Event};
pub use odra_types as types;
use types::{Address, RuntimeArgs, bytesrepr::{FromBytes, self}, CLType, CLTyped};
pub use variable::Variable;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_test_env::TestEnv;
        pub use odra_mock_env::ContractEnv;
        pub use odra_test_env::ContractContainer;
    } else if #[cfg(feature = "wasm")] {
        mod external_api;
        pub use external_api::env::ContractEnv;
    } else if #[cfg(feature = "wasm-test")] {
        mod external_api;
        pub use external_api::env::ContractEnv;
        pub use external_api::test_env::TestEnv;
        pub use odra_test_env::ContractContainer;
    }
}

pub fn call_contract<T>(
    address: &Address,
    entrypoint: &str,
    args: &RuntimeArgs,
) -> T
where T: CLTyped + FromBytes { 
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "mock-vm",feature = "wasm-test"))] {
            let has_return = CLType::Unit != T::cl_type();
            match TestEnv::call_contract(address, entrypoint, args, has_return) {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        }  else if #[cfg(feature = "wasm")] {
            let res = ContractEnv::call_contract(address, entrypoint, args);
            bytesrepr::deserialize(res).unwrap()
        } else {
            compile_error!("Unknown featue")
        }
    }
}