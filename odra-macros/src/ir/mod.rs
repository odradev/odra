//! The parsed input of the macros: what the user wrote, with every validation done and every
//! error reported, queried by the emitters in `crate::ast`.
//!
//! - [ModuleStructIR]: the `#[odra::module]` struct and its `#[odra::module(..)]` arguments.
//! - [ModuleImplIR]: the `#[odra::module]` impl block or trait, with its [FnIR] functions.
//! - [TypeIR]: the struct or enum of `#[odra::odra_type]` and `#[odra::odra_error]`.
//!
//! An IR is built from the parsed item with a `try_new`, which runs every check and combines
//! the errors ([errors::collect_all]); after that, its queries cannot fail on the user's input.

use convert_case::{Boundary, Case, Casing};
use proc_macro2::Ident;

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
pub use ty::{TypeIR, TypeKind, VariantIR};

const CONSTRUCTOR_NAME: &str = "init";
const UPGRADER_NAME: &str = "upgrade";
const FACTORY_NAME: &str = "new_contract";
const FACTORY_UPGRADE_NAME: &str = "upgrade_child_contract";
const BATCH_FACTORY_UPGRADE_NAME: &str = "batch_upgrade_child_contract";

/// What a function is to the contract, by its name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FnType {
    Constructor,
    Upgrader,
    Factory,
    FactoryUpgrader,
    FactoryBatchUpgrader,
    Regular
}

/// `__<module>_<part>`, the name of a module the macros generate next to `Module`: the module's
/// name in snake case, spanned as the module ident so an error inside points at it.
fn generated_mod_ident(module_ident: &Ident, part: &str) -> Ident {
    let snake_cased = module_ident
        .to_string()
        .from_case(Case::UpperCamel)
        .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit])
        .to_case(Case::Snake);
    Ident::new(&format!("__{snake_cased}_{part}"), module_ident.span())
}

#[cfg(test)]
mod tests;
