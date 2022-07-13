pub mod contract_def;
mod event;
pub mod instance;
mod mapping;
pub mod test_utils;
mod variable;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLType, CLTyped, RuntimeArgs};

pub use {
    mapping::Mapping,
    odra_proc_macros::{external_contract, instance, module, Event},
    odra_types as types,
    variable::Variable
};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_vm::{TestEnv, ContractEnv};
    } else if #[cfg(feature = "wasm")] {
        mod external_api;
        pub use external_api::env::ContractEnv;
    } else if #[cfg(feature = "wasm-test")] {
        mod external_api;
        pub use external_api::env::ContractEnv;
        pub use external_api::test_env::TestEnv;
    }
}

pub fn call_contract<T>(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> T
where
    T: CLTyped + FromBytes + Debug,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let has_return = CLType::Unit != T::cl_type();
            let result = TestEnv::call_contract(address, entrypoint, args, has_return);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        } else if #[cfg(feature = "wasm-test")] {
            let has_return = CLType::Unit != T::cl_type();
            let result = TestEnv::call_contract(address, entrypoint, args, has_return);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        }  else if #[cfg(feature = "wasm")] {
            let res = ContractEnv::call_contract(address, entrypoint, args);
            // TODO: Remove unwrap.
            types::bytesrepr::deserialize(res).unwrap()
        } else {
            compile_error!("Unknown feature")
        }
    }
}
