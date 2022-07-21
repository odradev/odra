use odra_codegen::generator::errors;
use proc_macro2::TokenStream;

pub(crate) fn generate_code(item: proc_macro::TokenStream) -> TokenStream {
    match syn::parse2::<syn::ItemEnum>(item.into()) {
        Ok(item_enum) => errors::generate_into_odra_error(item_enum),
        Err(err) => err.to_compile_error(),
    }
}
