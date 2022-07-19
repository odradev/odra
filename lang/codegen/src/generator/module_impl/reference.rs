use derive_more::From;
use odra_ir::module_item::{impl_item::ImplItem, module_impl::ModuleImpl};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{generator::common, GenerateCode};

#[derive(From)]
pub struct ContractReference<'a> {
    contract: &'a ModuleImpl,
}

as_ref_for_contract_impl_generator!(ContractReference);

impl GenerateCode for ContractReference<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let ref_ident = format_ident!("{}Ref", struct_ident);

        let methods = self.contract.methods();

        let ref_entrypoints = build_entrypoints(&methods);

        let ref_constructors = build_constructors(&methods);

        quote! {
            pub struct #ref_ident {
                address: odra::types::Address,
            }

            impl #ref_ident {
                #ref_entrypoints

                #ref_constructors

                pub fn address(&self) -> odra::types::Address {
                    self.address.clone()
                }

                pub fn at(address: odra::types::Address) -> Self {
                    Self { address }
                }
            }
        }
    }
}

fn build_entrypoints(methods: &[&ImplItem]) -> TokenStream {
    methods
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = &entrypoint.ident.to_string();
            let fn_body =
                common::generate_fn_body(entrypoint.args.clone(), entrypoint_name, &entrypoint.ret);

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}

fn build_constructors(methods: &[&ImplItem]) -> TokenStream {
    methods
        .iter()
        .filter_map(|item| match item {
            ImplItem::Constructor(constructor) => Some(constructor),
            _ => None,
        })
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = entrypoint.ident.to_string();
            let fn_body = common::generate_fn_body(
                entrypoint.args.clone(),
                &entrypoint_name,
                &syn::ReturnType::Default,
            );

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}
