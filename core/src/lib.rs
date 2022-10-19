mod instance;
mod list;
mod mapping;
mod unwrap_or_revert;
mod variable;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs, U512};

pub use {
    instance::Instance,
    list::List,
    mapping::Mapping,
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_types as types, odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable,
};

#[cfg(test)]
pub mod test_utils;

#[cfg(feature = "casper")]
pub use odra_casper_backend::contract_env;
#[cfg(feature = "casper-test")]
pub use odra_casper_test_env::test_env;
#[cfg(feature = "mock-vm")]
pub use odra_mock_vm::{contract_env, test_env};
#[cfg(feature = "casper-test")]
pub use odra_casper_test_env::dummy_contract_env as contract_env;

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(
    address: &Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>,
) -> T
where
    T: CLTyped + FromBytes + Debug,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let result = test_env::call_contract(address, entrypoint, args, amount);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        } else if #[cfg(feature = "casper-test")] {
            let has_return = types::CLType::Unit != T::cl_type();
            let result = test_env::call_contract(address, entrypoint, args, has_return, amount);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        }  else if #[cfg(feature = "casper")] {
            let res = contract_env::call_contract(address, entrypoint, args, amount);
            types::bytesrepr::deserialize(res).unwrap_or_revert()
        } else {
            compile_error!("Unknown feature")
        }
    }
}
