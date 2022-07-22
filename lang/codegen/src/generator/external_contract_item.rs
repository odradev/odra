use odra_ir::ExternalContractItem;
use proc_macro2::TokenStream;
use syn::parse_quote;

use super::common;

pub fn generate_code(item: ExternalContractItem) -> TokenStream {
    let ExternalContractItem {
        item_trait,
        item_ref,
    } = item;
    let trait_ident = &item_trait.ident;
    let ref_ident = &item_ref.ident;

    let methods = item_trait
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|item| {
            let sig = &item.sig;
            let entrypoint_name = &item.sig.ident.to_string();
            let args = common::filter_args(&item.sig.inputs);
            let ret = &sig.output;

            let fn_body = common::generate_fn_body(args, entrypoint_name, ret);
            let result: syn::ImplItemMethod = parse_quote! {
                #sig {
                    #fn_body
                }
            };
            result
        });

    quote::quote! {
        #item_trait

        pub struct #ref_ident {
            address: odra::types::Address,
        }

        impl #ref_ident {
            fn at(address: odra::types::Address) -> Self {
                Self { address }
            }

            fn address(&self) -> odra::types::Address {
                self.address.clone()
            }
        }

        impl #trait_ident for #ref_ident {
            # ( #methods) *
        }
    }
}
