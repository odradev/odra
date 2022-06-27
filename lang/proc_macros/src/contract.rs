use odra_ir::ContractItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match ContractItem::parse(attr.into(), item.into()) {
        Ok(contract) => match contract {
            ContractItem::Struct(item) => odra_codegen::generate_code(&item),
            ContractItem::Impl(item) => odra_codegen::generate_code(&item),
        },
        Err(err) => err.to_compile_error(),
    }
}
