use odra_ir::external_contract_item::ExternalContractItem;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro::TokenStream;
use odra_codegen::external_contract_item;


pub(crate) fn generate_code(attr: TokenStream, item: TokenStream) -> TokenStream2 {
    match ExternalContractItem::parse(attr.into(), item.into()) {
        Ok(item) => external_contract_item::generate_code(item),
        Err(err) => err.to_compile_error(),
    }
}