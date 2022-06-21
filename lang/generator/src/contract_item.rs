use odra_ir::contract_item::{
    contract_impl::ContractImpl, contract_struct::ContractStruct, ContractItem,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn generate_code(item: ContractItem) -> TokenStream {
    if let Some(item_struct) = item.contract_struct() {
        return generate_struct_code(item_struct);
    }

    let contract_impl = item.contract_impl().unwrap();
    generate_impl_code(contract_impl)
}

fn generate_struct_code(item_struct: &ContractStruct) -> TokenStream {
    item_struct.to_token_stream()
}

fn generate_impl_code(contract_impl: &ContractImpl) -> TokenStream {
    let entrypoints = contract_impl
        .entrypoints()
        .iter()
        .map(|entrypoint| entrypoint.to_token_stream())
        .flatten()
        .collect::<TokenStream>();

    let struct_ident = contract_impl.ident();
    let struct_name = struct_ident.to_string();
    let original_impl = contract_impl.original_impl();

    quote! {
        #original_impl

        impl odra::contract_def::HasContractDef for #struct_ident {
            fn contract_def() -> odra::contract_def::ContractDef {
                odra::contract_def::ContractDef {
                    ident: String::from(#struct_name),
                    entrypoints: vec![
                        #entrypoints
                    ],
                }
            }
        }
    }
}
