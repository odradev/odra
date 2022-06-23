use odra_ir::external_contract_item::ExternalContractItem;
use proc_macro2::TokenStream;

pub fn generate_code(item: ExternalContractItem) -> TokenStream {
    let item_trait = item.item_trait();
    quote::quote! {
        #item_trait
    }
}