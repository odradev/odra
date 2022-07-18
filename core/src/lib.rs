pub mod contract_def;
mod event;
pub mod instance;
mod mapping;
mod variable;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLType, CLTyped, RuntimeArgs, OdraError};

pub use {
    mapping::Mapping,
    odra_proc_macros::{external_contract, instance, module, Event},
    odra_types as types,
    odra_utils as utils,
    variable::Variable
};

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_vm::{TestEnv, ContractEnv};
        pub mod test_utils;
    } else if #[cfg(feature = "wasm-test")] {
        pub mod test_utils;
        mod external_api;
        pub use external_api::contract_env::ContractEnv;
        pub use external_api::test_env::TestEnv;
    } else if #[cfg(feature = "wasm")] {
        mod external_api;
        pub use external_api::contract_env::ContractEnv;
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
            types::bytesrepr::deserialize(res).unwrap_or_revert()
        } else {
            compile_error!("Unknown feature")
        }
    }
}

pub trait UnwrapOrRevert<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, err: E) -> T;

    fn unwrap_or_revert(self) -> T;
}

impl<T, E: Into<OdraError>> UnwrapOrRevert<T> for Result<T, E> {
    fn unwrap_or_revert_with<F: Into<OdraError>>(self, err: F) -> T {
        self.unwrap_or_else(|_| ContractEnv::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|err| ContractEnv::revert(err.into()))
    }
}

impl<T> UnwrapOrRevert<T> for Option<T> {
    fn unwrap_or_revert_with<E: Into<OdraError>>(self, err: E) -> T {
        self.unwrap_or_else(|| ContractEnv::revert(err.into()))
    }

    fn unwrap_or_revert(self) -> T {
        self.unwrap_or_else(|| ContractEnv::revert(OdraError::Unknown))
    }
}
