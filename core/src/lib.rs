#![feature(trait_alias)]

mod instance;
mod list;
mod mapping;
mod unwrap_or_revert;
mod variable;

pub use {
    instance::Instance,
    list::List,
    mapping::Mapping,
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable,
};

pub mod test_utils;

/*
    Environments import
*/
#[cfg(feature = "casper")]
pub use odra_casper_backend::contract_env;
#[cfg(feature = "casper-test")]
pub use odra_casper_test_env::dummy_contract_env as contract_env;
#[cfg(feature = "casper-test")]
pub use odra_casper_test_env::test_env;
use odra_mock_vm::types::{Address, Balance, CallArgs, OdraType};
#[cfg(feature = "mock-vm")]
pub use odra_mock_vm::{contract_env, test_env, types};

#[cfg(any(feature = "casper", feature = "casper-test"))]
use backend_casper::OdraType;

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(
    address: Address,
    entrypoint: &str,
    args: CallArgs,
    amount: Option<Balance>,
) -> T
where
    T: OdraType,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            test_env::call_contract(address, entrypoint, args, amount)
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
