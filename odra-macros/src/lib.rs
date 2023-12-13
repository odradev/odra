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

#[proc_macro_derive(OdraType)]
pub fn derive_odra_type(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        return OdraTypeItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or Enum expected")
}

#[proc_macro_derive(OdraError)]
pub fn derive_odra_error(item: TokenStream) -> TokenStream {
    let item = item.into();
    if let Ok(ir) = TypeIR::try_from(&item) {
        return OdraErrorItem::try_from(&ir).into_code();
    }
    span_error!(item, "Struct or Enum expected")
}

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
