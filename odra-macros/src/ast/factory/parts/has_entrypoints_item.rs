use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{ast::wasm_parts, ir::ModuleImplIR};

/// Emits `impl odra::contract_def::HasEntrypoints for XxxFactory`: a factory's entry points
/// are its argument-less constructor and the three factory functions, listed as
/// `odra::entry_point::EntryPoint` variants converted into contract definitions.
pub struct FactoryHasEntrypointsImplItem {
    module_ident: Ident,
    entry_points: Vec<syn::Expr>
}

impl TryFrom<&ModuleImplIR> for FactoryHasEntrypointsImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let constructor = module
            .constructor()
            .map(|_| wasm_parts::constructor_ep(syn::parse_quote!(odra::prelude::vec![])));
        Ok(Self {
            module_ident: module.module_ident(),
            entry_points: constructor
                .into_iter()
                .chain([
                    wasm_parts::factory_ep(wasm_parts::parameters(&module.factory_fn())),
                    wasm_parts::factory_upgrade_ep(wasm_parts::parameters(
                        &module.factory_upgrade_fn()
                    )),
                    wasm_parts::factory_batch_upgrade_ep()
                ])
                .collect()
        })
    }
}

impl ToTokens for FactoryHasEntrypointsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let entry_points = &self.entry_points;

        tokens.extend(quote! {
            impl odra::contract_def::HasEntrypoints for #module_ident {
                fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                    odra::prelude::vec![#(#entry_points),*]
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_, _>>()
                        .unwrap_or_default()
                }
            }
        });
    }
}

#[cfg(test)]
mod tests;
