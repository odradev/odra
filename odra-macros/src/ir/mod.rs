//! The parsed input of the macros: what the user wrote, with every validation done and every
//! error reported, queried by the emitters in `crate::ast`.
//!
//! - [ModuleStructIR]: the `#[odra::module]` struct and its `#[odra::module(..)]` arguments.
//! - [ModuleImplIR]: the `#[odra::module]` impl block or trait, with its [FnIR] functions.
//! - [TypeIR]: the struct or enum of `#[odra::odra_type]` and `#[odra::odra_error]`.
//!
//! An IR is built with `TryFrom`, which runs every check and combines the errors
//! ([errors::collect_all]); after that, its queries cannot fail on the user's input.

mod attr;
mod config;
pub mod delegate;
pub mod errors;
mod function;
mod module_impl;
mod module_struct;
mod ty;

pub use function::{FnArgIR, FnIR};
pub use module_impl::ModuleImplIR;
pub use module_struct::{EnumeratedTypedField, ModuleStructIR};
pub use ty::{TypeIR, TypeKind};

const CONSTRUCTOR_NAME: &str = "init";
const UPGRADER_NAME: &str = "upgrade";
const FACTORY_NAME: &str = "new_contract";
const FACTORY_UPGRADE_NAME: &str = "upgrade_child_contract";
const BATCH_FACTORY_UPGRADE_NAME: &str = "batch_upgrade_child_contract";

/// What a function is to the contract, by its name.
#[derive(PartialEq, Eq)]
pub enum FnType {
    Constructor,
    Upgrader,
    Factory,
    FactoryUpgrader,
    FactoryBatchUpgrader,
    Regular
}

/// Parses the attribute arguments and the item a `#[odra::module]`-like macro was applied to.
fn parse_attr_and_item<T: syn::parse::Parse>(
    stream: (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
) -> syn::Result<(config::ConfigItem, T)> {
    Ok((
        syn::parse2::<config::ConfigItem>(stream.0.clone())?,
        syn::parse2::<T>(stream.1.clone())?
    ))
}

#[cfg(test)]
mod tests;
