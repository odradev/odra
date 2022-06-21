use odra_codegen::event_item;
use odra_ir::event_item::EventItem;
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn generate_code(input: DeriveInput) -> TokenStream {
    match EventItem::parse(input) {
        Ok(item) => event_item::generate_code(item),
        Err(err) => err.to_compile_error(),
    }
}
