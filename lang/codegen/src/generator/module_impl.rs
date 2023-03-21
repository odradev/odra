use derive_more::From;
use odra_ir::module::ImplItem;
use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{poet::OdraPoetUsingImpl, GenerateCode};

use self::{def::ContractDef, deploy::Deploy, reference::ContractReference};

mod def;
mod deploy;
mod reference;

#[derive(From)]
pub struct ModuleImpl<'a> {
    contract: &'a odra_ir::module::ModuleImpl
}

as_ref_for_contract_impl_generator!(ModuleImpl);

impl GenerateCode for ModuleImpl<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let ident = self.contract.ident();
        let original_item_impls = self.contract.impl_items().iter().map(|item| match item {
            ImplItem::Constructor(item) => item.impl_item.to_token_stream(),
            ImplItem::Method(item) => item.impl_item.to_token_stream(),
            ImplItem::Other(item) => item.to_token_stream(),
            ImplItem::DelegatedMethod(_) => todo!(),
        });

        let contract_def = self.generate_code_using::<ContractDef>();
        let deploy = self.generate_code_using::<Deploy>();
        let contract_ref = self.generate_code_using::<ContractReference>();

        let delegated_functions = self
            .contract
            .delegation_stmts()
            .iter()
            .map(|stmt| {
                let expr = &stmt.delegate_to;
                let block = &stmt.delegation_block;
                block
                    .functions
                    .iter()
                    .map(|f| {
                        let vis = &f.visibility;
                        let fn_item = &f.fn_item;
                        let sig = fn_item.sig.clone();

                        let ident = sig.ident.clone();

                        let args = fn_item
                            .sig
                            .inputs
                            .iter()
                            .filter_map(|arg| match arg {
                                syn::FnArg::Receiver(_) => None,
                                syn::FnArg::Typed(arg) => Some(arg.pat.clone())
                            })
                            .collect::<Vec<_>>();
                        quote::quote! {
                            #vis #sig {
                                #expr.#ident(#(#args),*)
                            }
                        }
                    })
                    .collect::<TokenStream>()
            })
            .collect::<TokenStream>();

        quote::quote! {
            impl #ident {
                # (#original_item_impls)*

                #delegated_functions
            }

            #[cfg(feature = "casper")]
            #contract_def

            #deploy

            #contract_ref
        }
    }
}
