use odra_ir::EventItem;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn generate_code(input: DeriveInput) -> TokenStream {
    match EventItem::parse(input) {
        Ok(item) => odra_codegen::generate_code(&item),
        Err(err) => err.to_compile_error(),
    }
}
