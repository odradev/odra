use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::ir::ModuleStructIR;

/// Emits `impl odra::contract_def::HasIdent for Xxx`: `ident()` is the struct's name and
/// `contract_name()` the `name = ".."` of `#[odra::module(..)]`, present only when it was given.
pub struct HasIdentImplItem {
    module_ident: syn::Ident,
    contract_name: Option<String>
}

impl TryFrom<&ModuleStructIR> for HasIdentImplItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
            contract_name: ir.contract_name()
        })
    }
}

impl ToTokens for HasIdentImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let module_name = module_ident.to_string();
        let contract_name_fn = self.contract_name.as_ref().map(|name| {
            quote! {
                fn contract_name() -> odra::prelude::string::String {
                    odra::prelude::string::String::from(#name)
                }
            }
        });

        tokens.extend(quote! {
            impl odra::contract_def::HasIdent for #module_ident {
                fn ident() -> odra::prelude::string::String {
                    odra::prelude::string::String::from(#module_name)
                }
                #contract_name_fn
            }
        });
    }
}

#[cfg(test)]
mod tests;
