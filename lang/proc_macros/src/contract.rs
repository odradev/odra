use odra_codegen::contract_item;
use odra_ir::contract_item::ContractItem;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

pub fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match ContractItem::parse(attr.into(), item.into()) {
        Ok(contract) => contract_item::generate_code(contract),
        Err(err) => err.to_compile_error(),
    }
}
