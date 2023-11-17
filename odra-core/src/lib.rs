#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(once_cell)]

extern crate alloc;

pub mod call_result;
mod contract_context;
mod contract_env;
pub mod entry_point_callback;
pub mod event;
mod host_context;
mod host_env;
mod key_maker;
pub mod mapping;
pub mod module;
mod odra_result;
pub mod prelude;
pub mod variable;

pub use casper_event_standard;
pub use contract_context::ContractContext;
pub use contract_env::ContractEnv;
pub use entry_point_callback::EntryPointsCaller;
pub use host_context::HostContext;
pub use host_env::HostEnv;
pub use module::ModuleCaller;
pub use odra_result::OdraResult;
pub use odra_types::call_def::CallDef;
