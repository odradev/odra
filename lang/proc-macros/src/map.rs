use odra_ir::MapExpr;
use proc_macro2::TokenStream;

pub(crate) fn generate_code(item: proc_macro::TokenStream) -> TokenStream {
    match syn::parse2::<MapExpr>(item.into()) {
        Ok(item_enum) => odra_codegen::generate_code(&item_enum),
        Err(err) => err.to_compile_error()
    }
}
