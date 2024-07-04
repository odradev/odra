#![doc = "Core of the Odra Framework"]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(never_type))]

extern crate alloc;

mod address;
pub mod args;
pub mod arithmetic;
mod call_def;
mod call_result;
pub mod callstack;
#[doc(hidden)]
pub mod consts;
mod contract_container;
mod contract_context;
pub mod contract_def;
mod contract_env;
mod contract_register;
pub mod crypto;
pub mod entry_point_callback;
mod error;
mod external;
mod gas_report;
pub mod host;
mod list;
mod mapping;
pub mod module;
mod named_keys;
pub mod prelude;
mod sequence;
pub mod uints;
mod unwrap_or_revert;
pub mod utils;
mod var;

pub use address::{Address, Addressable};
pub use call_def::CallDef;
pub use call_result::ContractCallResult;
pub use casper_event_standard;
pub use contract_container::ContractContainer;
pub use contract_context::ContractContext;
pub use contract_env::{ContractEnv, ContractRef, ExecutionEnv};
pub use contract_register::ContractRegister;
pub use error::{
    AddressError, CollectionError, EventError, ExecutionError, OdraError, OdraResult, VmError
};
pub use unwrap_or_revert::UnwrapOrRevert;

pub use external::External;
pub use gas_report::*;
pub use list::{List, ListIter};
pub use mapping::Mapping;
pub use module::{Module, SubModule};
pub use sequence::Sequence;
pub use var::Var;

pub use casper_types;
