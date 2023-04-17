#[cfg(all(feature = "casper", feature = "mock-vm"))]
compile_error!("casper and mock-vm are mutually exclusive features.");

#[cfg(not(any(feature = "casper", feature = "mock-vm")))]
compile_error!("Exactly one of these features must be selected: `casper`, `mock-vm`.");

mod instance;
mod is_module;
mod list;
mod mapping;
mod unwrap_or_revert;
mod variable;

pub use {
    instance::Instance,
    is_module::IsModule,
    list::List,
    mapping::Mapping,
    odra_proc_macros::{
        execution_error, external_contract, map, module, odra_error, Event, Instance
    },
    odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable
};

#[cfg(not(target_arch = "wasm32"))]
pub mod test_utils;

#[cfg(all(feature = "casper", target_arch = "wasm32"))]
pub use odra_casper_backend::contract_env;
#[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
pub use odra_casper_test_env::{dummy_contract_env as contract_env, test_env};
#[cfg(feature = "mock-vm")]
pub use odra_mock_vm::{contract_env, test_env};

/// Types that are used in Odra framework.
///
/// Re-exports all the platform-agnostic types and depending on the selected feature, re-exports platform-specific types.
pub mod types {
    #[cfg(feature = "casper")]
    pub use odra_casper_backend::types::*;
    #[cfg(feature = "mock-vm")]
    pub use odra_mock_vm::types::*;
    pub use odra_types::*;
}

#[cfg(feature = "casper")]
pub mod casper {
    pub use odra_casper_backend::casper_types;
    #[cfg(target_arch = "wasm32")]
    pub use odra_casper_backend::{casper_contract, runtime, storage, utils};
    #[cfg(not(target_arch = "wasm32"))]
    pub use odra_casper_codegen as codegen;
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(
    address: types::Address,
    entrypoint: &str,
    args: types::CallArgs,
    amount: Option<types::Balance>
) -> T
where
    T: types::OdraType
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            test_env::call_contract(address, entrypoint, args, amount)
        } else if #[cfg(all(feature = "casper", not(target_arch = "wasm32")))] {
            test_env::call_contract(address, entrypoint, args, amount)
        }  else if #[cfg(all(feature = "casper", target_arch = "wasm32"))] {
            contract_env::call_contract(address, entrypoint, args, amount)
        } else {
            compile_error!("Unknown feature")
        }
    }
}
