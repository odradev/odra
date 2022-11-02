use odra_ir::ErrorEnumItem;
use proc_macro2::TokenStream;

pub(crate) fn generate_code(item: proc_macro::TokenStream) -> TokenStream {
    match ErrorEnumItem::parse(item.into()) {
        Ok(item) => odra_codegen::generate_code(&item),
        Err(err) => err.to_compile_error()
    }
}
