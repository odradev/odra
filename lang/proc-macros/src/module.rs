use odra_ir::module::ModuleItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match ModuleItem::parse(attr.into(), item.into()) {
        Ok(contract) => match contract {
            ModuleItem::Struct(item) => odra_codegen::generate_code(item.as_ref()),
            ModuleItem::Impl(item) => odra_codegen::generate_code(item.as_ref())
        },
        Err(err) => err.to_compile_error()
    }
}
