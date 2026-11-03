//! The code the macros emit, one item type per generated piece.
//!
//! An item is built from the parsed input (`crate::ir`) and written out by `ToTokens`. The
//! top-level items (`ModuleStructItem`, `ModuleImplItem`, ...) are structs whose fields are the
//! generated pieces in emission order, composed with `derive_try_from_ref::TryFromRef`.
//!
//! Every item file follows one template:
//!
//! - A `/// Emits ...` doc on the item struct naming the Rust item it produces.
//! - Fields hold data: idents, types, strings and values derived from the IR. A pre-built syn
//!   node (an `ItemFn`, an `Expr`) is a field only when a helper builds it for more than one
//!   emitter, such as the module and the factory host refs. A piece used once is built inside
//!   `to_tokens`.
//! - `impl TryFrom<&XxxIR>` does every validation and raises every error (built in
//!   `crate::ir::errors`); it never formats code.
//! - `impl ToTokens` is one `quote!` per emitted item, with the `let` bindings it needs right
//!   above it. Paths into the `odra` crate come from `crate::utils::ty` and `crate::utils::ident`,
//!   so a path is spelled in one place.
//! - Spans: a token copied from the user's code (an ident, a type, an attribute) is reused as its
//!   syn node so it keeps the user's span and rustc points at it; a token the macro invents has
//!   the call-site span. `quote_spanned!` is used only where an invented token must point at a
//!   user item, such as a bound check on an argument type.
//! - Snapshot tests live next to the file in `<file>/tests.rs`, declared as
//!   `#[cfg(test)] mod tests;`, and compare the emitted code with a `quote!` block.
mod blueprint;
mod contract_item;
mod contract_ref_item;
mod deployer_item;
mod entrypoints_item;
mod error_item;
mod event;
mod events_item;
mod exec_parts;
mod external_contract_item;
mod factory;
mod fn_utils;
mod host_ref_item;
mod ident_item;
mod module_def;
mod module_impl_item;
mod module_item;
mod module_struct_item;
mod odra_type_item;
mod parts_utils;
mod ref_helpers_item;
mod ref_utils;
mod schema;
mod test_parts;
mod utils;
mod wasm_parts;

pub(crate) use error_item::OdraErrorAttrItem;
pub(crate) use event::OdraEventItem;
pub(crate) use external_contract_item::ExternalContractImpl;
pub(crate) use factory::impl_item::FactoryModuleImplItem;
pub(crate) use factory::struct_item::FactoryModuleStructItem;
pub(crate) use module_impl_item::ModuleImplItem;
pub(crate) use module_struct_item::ModuleStructItem;
pub(crate) use odra_type_item::OdraTypeAttrItem;
pub(crate) use ref_helpers_item::RefHelpersItem;
