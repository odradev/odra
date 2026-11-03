use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{ast::wasm_parts, ir::ModuleImplIR, utils};

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
            .map(|_| utils::expr::constructor_ep(syn::parse_quote!(odra::prelude::vec![])));
        Ok(Self {
            module_ident: module.module_ident()?,
            entry_points: constructor
                .into_iter()
                .chain([
                    utils::expr::factory_ep(wasm_parts::parameters(&module.factory_fn())),
                    utils::expr::factory_upgrade_ep(wasm_parts::parameters(
                        &module.factory_upgrade_fn()
                    )),
                    utils::expr::factory_batch_upgrade_ep()
                ])
                .collect()
        })
    }
}

impl ToTokens for FactoryHasEntrypointsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let module_ident = &self.module_ident;
        let entry_points = &self.entry_points;
        let ty_has_entrypoints = utils::ty::has_entrypoints();
        let ty_entrypoint = utils::ty::entry_point_def();
        let entrypoints = utils::ident::entrypoints();

        tokens.extend(quote! {
            impl #ty_has_entrypoints for #module_ident {
                fn #entrypoints() -> odra::prelude::vec::Vec<#ty_entrypoint> {
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
