#![feature(box_patterns)]

use ast::*;
use ir::ModuleIR;
use proc_macro::TokenStream;

mod ast;
mod ir;
#[cfg(test)]
mod test_utils;
mod utils;

#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match module_impl(item) {
        Ok(result) => result,
        Err(e) => e.to_compile_error()
    }
    .into()
}

fn module_impl(item: TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let module_ir = ModuleIR::try_from(&item.into())?;

    let code = module_ir.self_code();
    let ref_item = RefItem::try_from(&module_ir)?;
    let test_parts = TestParts::try_from(&module_ir)?;

    Ok(quote::quote! {
        #code
        #ref_item
        #test_parts
    })
}
