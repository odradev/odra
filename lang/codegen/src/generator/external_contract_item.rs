use derive_more::From;
use odra_ir::ExternalContractItem as IrExternalContractItem;
use syn::parse_quote;

use super::common::{self, build_ref};
use crate::GenerateCode;

#[derive(From)]
pub struct ExternalContractItem<'a> {
    item: &'a IrExternalContractItem
}

impl GenerateCode for ExternalContractItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let IrExternalContractItem {
            item_trait,
            item_ref
        } = &self.item;
        let trait_ident = &item_trait.ident;
        let ref_ident = &item_ref.ident;

        let methods = item_trait
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Method(method) => Some(method),
                _ => None
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
        let contract_ref = build_ref(ref_ident, trait_ident);

        quote::quote! {
            #item_trait

            #contract_ref

            impl #trait_ident for #ref_ident {
                # ( #methods) *
            }
        }
    }
}
