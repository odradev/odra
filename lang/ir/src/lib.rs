//! Odra intermediate representation (IR) and abstractions.
//!
//! This module defines everything Odra procedural macro needs in order to parse,
//! analyze and generate code for smart contracts.
//!
//! This crate takes care of parsing and analyzing code. This process may fail
//! return [`syn::Error`].
//!
//! All the items are based on syn with special variants for Odra `impl` items.

mod attrs;
mod event_item;
mod execution_error;
mod external_contract_item;
mod instance_item;
mod module_item;

pub use {
    event_item::EventItem, execution_error::error_enum::ErrorEnumItem,
    external_contract_item::ExternalContractItem, instance_item::InstanceItem
};

/// Odra module-related abstractions.
pub mod module {
    pub use crate::module_item::{
        constructor::Constructor, impl_item::ImplItem, method::Method, module_impl::ModuleImpl,
        module_struct::ModuleStruct, ModuleItem
    };
}
