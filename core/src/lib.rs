#![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(never_type))]

extern crate alloc;

mod address;
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
pub mod host;
mod list;
mod mapping;
pub mod module;
pub mod prelude;
mod sequence;
pub mod uints;
mod unwrap_or_revert;
pub mod utils;
mod variable;

pub use address::Address;
pub use call_def::CallDef;
pub use call_result::ContractCallResult;
pub use casper_event_standard;
pub use contract_container::ContractContainer;
pub use contract_context::ContractContext;
pub use contract_env::{ContractEnv, ExecutionEnv};
pub use contract_register::ContractRegister;
pub use error::{
    AddressError, CollectionError, EventError, ExecutionError, OdraError, OdraResult, VmError
};
pub use unwrap_or_revert::UnwrapOrRevert;

pub use list::{List, ListIter};
pub use mapping::Mapping;
pub use sequence::Sequence;
pub use variable::Variable;

pub use casper_types;
