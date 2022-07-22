pub mod contract_def;
pub mod instance;
mod mapping;
mod unwrap_or_revert;
mod variable;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs};

pub use {
    mapping::Mapping,
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_types as types, odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable,
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

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> T
where
    T: CLTyped + FromBytes + Debug,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let has_return = types::CLType::Unit != T::cl_type();
            let result = TestEnv::call_contract(address, entrypoint, args, has_return);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        } else if #[cfg(feature = "wasm-test")] {
            let has_return = types::CLType::Unit != T::cl_type();
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
