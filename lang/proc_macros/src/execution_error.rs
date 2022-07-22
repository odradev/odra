use odra_codegen::generator::errors;
use odra_ir::ErrorEnumItem;
use proc_macro2::TokenStream;

pub(crate) fn generate_code(item: proc_macro::TokenStream) -> TokenStream {
    match ErrorEnumItem::parse(item.into()) {
        Ok(item) => errors::generate_error_enum(item),
        Err(err) => err.to_compile_error(),
    }
}
