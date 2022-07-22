use odra_codegen::generator::instance_item;
use odra_ir::InstanceItem;
use proc_macro2::TokenStream as TokenStream2;
use syn::DeriveInput;

pub fn generate_code(input: DeriveInput) -> TokenStream2 {
    match InstanceItem::parse(input) {
        Ok(item) => instance_item::generate_code(item),
        Err(err) => err.to_compile_error(),
    }
}
