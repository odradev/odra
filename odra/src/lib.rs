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
    args, contract_def, entry_point_callback, host, module, prelude, uints, utils
};
/// Macros for creating Odra modules that store values in named keys or dictionaries.
pub mod named_keys {
    pub use odra_core::{
        base64_encoded_key_value_storage, compound_key_value_storage, key_value_storage,
        single_value_storage
    };
}
pub use odra_core::{casper_event_standard, casper_event_standard::Event, casper_types};
pub use odra_core::{
    AddressError, Addressable, CallDef, CollectionError, ContractCallResult, ContractContext,
    ContractEnv, ContractRef, DeployReport, EventError, ExecutionEnv, GasReport, OdraContract,
    VmError
};

pub use odra_macros::*;
#[cfg(not(target_arch = "wasm32"))]
pub use odra_schema as schema;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_wasm_env;
