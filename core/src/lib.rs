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

#[cfg(not(target_arch = "wasm32"))]
pub mod test_utils;

#[cfg(feature = "casper")]
pub use odra_casper_backend::types;
#[cfg(all(feature = "casper", target_arch = "wasm32"))]
pub use odra_casper_backend::contract_env;
#[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
pub use odra_casper_test_env::{test_env, dummy_contract_env as contract_env};
#[cfg(feature = "mock-vm")]
pub use odra_mock_vm::{contract_env, test_env, types};


/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(
    address: types::Address,
    entrypoint: &str,
    args: types::CallArgs,
    amount: Option<types::Balance>,
) -> T
where
    T: types::OdraType,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            test_env::call_contract(address, entrypoint, args, amount)
        } else if #[cfg(all(feature = "casper", not(target_arch = "wasm32")))] {
           test_env::call_contract(address, entrypoint, args, amount)
        }  else if #[cfg(all(feature = "casper", target_arch = "wasm32"))] {
            let res = contract_env::call_contract(address, entrypoint, args, amount);
            types::bytesrepr::deserialize(res).unwrap_or_revert()
        } else {
            compile_error!("Unknown feature")
        }
    }
}
