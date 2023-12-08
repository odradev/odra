#![feature(box_patterns, result_flattening)]

use ast::*;
use derive_try_from::TryFromRef;
use ir::{ModuleIR, StructIR, TypeIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

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
    if let Ok(ir) = ModuleIR::try_from((&attr, &item)) {
        return ModuleImpl::try_from(&ir).into_code();
    }
    if let Ok(ir) = StructIR::try_from((&attr, &item)) {
        return ModuleStruct::try_from(&ir).into_code();
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

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
struct ModuleImpl {
    #[expr(item.self_code())]
    self_code: syn::ItemImpl,
    has_entrypoints_item: HasEntrypointsImplItem,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ExecPartsItem,
    exec_parts_reexport: ExecPartsReexportItem,
    wasm_parts: WasmPartsModuleItem,
    blueprint: BlueprintItem
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(StructIR)]
struct ModuleStruct {
    #[expr(item.self_code().clone())]
    self_code: syn::ItemStruct,
    mod_item: ModuleModItem,
    has_ident_item: HasIdentImplItem,
    has_events_item: HasEventsImplItem
}

trait IntoCode {
    fn into_code(self) -> TokenStream;
}

impl<T: ToTokens> IntoCode for Result<T, syn::Error> {
    fn into_code(self) -> TokenStream {
        match self {
            Ok(data) => data.to_token_stream(),
            Err(e) => e.to_compile_error()
        }
        .into()
    }
}
