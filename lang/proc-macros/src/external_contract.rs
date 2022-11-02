use odra_ir::ExternalContractItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match ExternalContractItem::parse(attr.into(), item.into()) {
        Ok(item) => odra_codegen::generate_code(&item),
        Err(err) => err.to_compile_error()
    }
}