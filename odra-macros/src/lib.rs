#![feature(box_patterns, result_flattening)]

use ast::*;
use derive_try_from::TryFromRef;
use ir::{ModuleIR, StructIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::spanned::Spanned;

mod ast;
mod ir;
#[cfg(test)]
mod test_utils;
mod utils;

#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let stream: TokenStream2 = item.into();
    if let Ok(ir) = ModuleIR::try_from(&stream) {
        return ModuleImpl::try_from(&ir).into_code();
    }
    if let Ok(ir) = StructIR::try_from(&stream) {
        return ModuleStruct::try_from(&ir).into_code();
    }
    syn::Error::new(stream.span(), "Struct or impl block expected")
        .to_compile_error()
        .into()
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
struct ModuleImpl {
    #[expr(item.self_code().clone())]
    self_code: syn::ItemImpl,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem,
    exec_parts: ExecPartsItem,
    exec_parts_reexport: ExecPartsReexportItem,
    wasm_parts: WasmPartsModuleItem
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(StructIR)]
struct ModuleStruct {
    #[expr(item.self_code().clone())]
    self_code: syn::ItemStruct,
    mod_item: ModuleModItem
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
