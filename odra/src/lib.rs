//! A Rust library for writing smart contracts for the Casper Blockchain.
//!
//! # Example
//!
//! The following example is a simple counter smart contract.
//! The contract stores a single value, which can be incremented or decremented.
//! The counter value can be initialized at contract creation time.
//!
//! ```no_run
//! use odra::prelude::*;
//!
//! #[odra::module]
//! struct Counter {
//!     count: Var<u32>,
//! }
//!
//! #[odra::module]
//! impl Counter {
//!     pub fn init(&mut self, count: u32) {
//!         self.count.set(count);
//!     }
//!
//!     pub fn increment(&mut self) {
//!         self.count.set(self.count.get_or_default() + 1);
//!     }
//!
//!     pub fn decrement(&mut self) {
//!         self.count.set(self.count.get_or_default() - 1);
//!     }
//!
//!     pub fn get_count(&self) -> u32 {
//!         self.count.get_or_default()
//!     }
//! }
//!
//! # fn main() {}
//! ```
#![no_std]

pub use odra_core::{
    args, consts, contract_def, entry_point, entry_point_callback, host, module, prelude, uints,
    utils
};
/// Macros for creating Odra modules that store values in named keys or dictionaries.
pub mod named_keys {
    pub use odra_core::{
        base64_encoded_key_value_storage, compound_key_value_storage, dictionary_value_storage,
        key_value_storage, single_value_storage
    };
}
pub use odra_core::{casper_event_standard, casper_event_standard::Event, casper_types};
pub use odra_core::{
    AddressError, CallDef, CollectionError, ContractAddress, ContractCallResult, ContractContext,
    ContractEnv, ContractRef, DeployReport, EventError, ExecutionEnv, GasReport, OdraContract,
    VmError
};

pub use odra_macros::*;

/// Links the dependency crates that define contracts listed in `Odra.toml` into a binary.
///
/// Rust links a dependency only into a crate that refers to it, so a `bin/build_contract.rs` that
/// builds `odra_modules::erc20::Erc20` without mentioning `odra_modules` would make a wasm
/// without entry points. The macro expands to an `extern crate` item per such dependency, from
/// the file `odra_build::build()` writes in the build script, so the binaries of a project follow
/// its `Odra.toml` without being edited:
///
/// ```ignore
/// // bin/build_contract.rs and bin/build_schema.rs
/// use my_project;
///
/// odra::link_contract_crates!();
/// ```
///
/// The crate needs the `odra_build::build()` build script, as every crate with those binaries has.
/// Optional and target-specific dependencies are not covered; name those with a `use`.
#[macro_export]
macro_rules! link_contract_crates {
    () => {
        include!(concat!(env!("OUT_DIR"), "/odra_contract_crates.rs"));
    };
}
#[cfg(not(target_arch = "wasm32"))]
pub use odra_schema as schema;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_wasm_env;
