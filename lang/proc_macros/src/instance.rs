use odra_codegen::generator::instance_item;
use odra_ir::InstanceItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match InstanceItem::parse(attr.into(), item.into()) {
        Ok(item) => instance_item::generate_code(item),
        Err(err) => err.to_compile_error(),
    }
}
