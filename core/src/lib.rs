#![cfg_attr(not(any(feature = "wasm-test", feature = "mock-vm", test)), no_std)]
extern crate alloc;

#[cfg(any(feature = "wasm-test", feature = "mock-vm", test))]
extern crate std;

#[cfg(all(feature = "wasm-test", feature = "mock-vm"))]
compile_error!("wasm-test and mock-vm are mutually exclusive features.");
#[cfg(all(feature = "wasm-test", feature = "wasm"))]
compile_error!("wasm-test and wasm are mutually exclusive features.");
#[cfg(all(feature = "wasm", feature = "mock-vm"))]
compile_error!("wasm and mock-vm are mutually exclusive features.");
#[cfg(all(feature = "wasm", feature = "wasm-test", feature = "mock-vm"))]
compile_error!("wasm, wasm-test, mock-vm are mutually exclusive features.");

#[cfg(not(any(feature = "wasm-test", feature = "mock-vm", feature = "wasm")))]
compile_error!("Exactly one of these features must be selected: `wasm-test`, `mock-vm`, `wasm`.");

pub mod contract_def;
mod instance;
mod list;
mod mapping;
mod unwrap_or_revert;
mod variable;

use types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs};

pub use {
    instance::Instance,
    list::List,
    mapping::Mapping,
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_types as types, odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable,
};

#[cfg(feature = "external-api")]
mod external_api;
#[cfg(feature = "test-support")]
pub mod test_utils;
#[cfg(feature = "external-api")]
pub use external_api::contract_env::ContractEnv;

cfg_if::cfg_if! {
    if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_vm::{TestEnv, ContractEnv};
    } else if #[cfg(feature = "wasm-test")] {
        pub use external_api::test_env::TestEnv;
    }
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> T
where
    T: CLTyped + FromBytes,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let result = TestEnv::call_contract(address, entrypoint, args);
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
