#![feature(box_patterns, result_flattening)]

use ast::*;
use ir::{ModuleIR, StructIR};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
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
        return handle_result(module_impl(ir));
    }
    if let Ok(ir) = StructIR::try_from(&stream) {
        return handle_result(module_struct(ir));
    }
    handle_result(Err(syn::Error::new(
        stream.span(),
        "Struct or impl block expected"
    )))
}

fn module_impl(ir: ModuleIR) -> Result<TokenStream2, syn::Error> {
    let code = ir.self_code();
    let ref_item = RefItem::try_from(&ir)?;
    let test_parts = TestParts::try_from(&ir)?;
    let test_parts_reexport = TestPartsReexport::try_from(&ir)?;
    let wasm_parts = WasmPartsModuleItem::try_from(&ir)?;

    Ok(quote::quote! {
        #code
        #ref_item
        #test_parts
        #test_parts_reexport
        #wasm_parts
    })
}

fn module_struct(ir: StructIR) -> Result<TokenStream2, syn::Error> {
    let code = ir.self_code();
    let module_mod = ModuleModItem::try_from(&ir)?;

    Ok(quote::quote!(
        #code
        #module_mod
    ))
}

fn handle_result(result: Result<TokenStream2, syn::Error>) -> TokenStream {
    match result {
        Ok(stream) => stream,
        Err(e) => e.to_compile_error()
    }
    .into()
}
