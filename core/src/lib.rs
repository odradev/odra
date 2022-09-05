pub mod contract_def;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs};

pub use {
    odra_primitives::{instance::Instance, mapping::Mapping, variable::Variable},
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_types as types, odra_utils as utils,
    odra_contract_env::{ContractEnv, unwrap_or_revert::UnwrapOrRevert}
};

#[cfg(any(feature = "mock-vm", feature = "wasm-test"))]
pub use odra_contract_env::{TestEnv, test_utils};

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
