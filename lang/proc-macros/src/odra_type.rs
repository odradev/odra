use odra_ir::OdraTypeItem;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn generate_code(input: DeriveInput) -> TokenStream {
    match OdraTypeItem::parse(input) {
        Ok(item) => odra_codegen::generate_code(&item),
        Err(err) => err.to_compile_error()
    }
}
