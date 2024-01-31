#![feature(box_patterns, result_flattening)]

use crate::utils::IntoCode;
use ast::*;
use ir::{ModuleImplIR, ModuleStructIR, TypeIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod ast;
mod ir;
#[cfg(test)]
mod test_utils;
mod utils;

macro_rules! span_error {
    ($span:ident, $msg:expr) => {
        syn::Error::new(syn::spanned::Spanned::span(&$span), $msg)
            .to_compile_error()
            .into()
    };
}

/// Core element of the Odra framework, entry point for writing smart contracts.
///
/// Each module consists of two parts:
/// 1. Module definition - a struct which composition of stored values (Variables and Mappings)
/// and modules.
/// 2. Module implementation - an implementation block.
///
/// The macro produces all the required code to use the module as a standalone smart contract.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();
    if let Ok(ir) = ModuleImplIR::try_from((&attr, &item)) {
        return ModuleImplItem::try_from(&ir).into_code();
    }
    if let Ok(ir) = ModuleStructIR::try_from((&attr, &item)) {
        return ModuleStructItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or impl block expected")
}

/// Implements boilerplate for a type to be used in an Odra module.
///
/// This macro implements serialization and deserialization for the type, as well as
/// cloning and [HasEvents](../odra_core/contract_def/trait.HasEvents.html) trait.
#[proc_macro_derive(OdraType)]
pub fn derive_odra_type(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        return OdraTypeItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or Enum expected")
}

/// Implements `Into<odra::OdraError>` for an error enum.
#[proc_macro_derive(OdraError)]
pub fn derive_odra_error(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        return OdraErrorItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or Enum expected")
}

/// Provides implementation of a reference to an external contract.
///
/// If you don't have access to the contract source code, but want to call it,
/// you can create a reference to it and interact exactly the same way as with a contract
/// written using [macro@module] macro.
#[proc_macro_attribute]
pub fn external_contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item: TokenStream2 = item.into();
    if let Ok(ir) = ModuleImplIR::try_from((&attr, &item)) {
        return ExternalContractImpl::try_from(&ir).into_code();
    }
    span_error!(
        item,
        "#[external_contract] can be only applied to trait only"
    )
}
