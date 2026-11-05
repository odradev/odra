use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::wasm_parts,
    ir::{FnIR, FnType, ModuleImplIR},
    utils
};

/// Emits `impl odra::contract_def::HasEntrypoints for Xxx`: `entrypoints()` lists every entry
/// point as an `odra::contract_def::Entrypoint`, the contract definition the host and the
/// schema read.
pub struct HasEntrypointsImplItem {
    module_ident: Ident,
    entry_points: Vec<EntrypointDef>
}

impl TryFrom<&ModuleImplIR> for HasEntrypointsImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident(),
            entry_points: module
                .onchain_functions()
                .into_iter()
                .map(EntrypointDef::try_new)
                .collect::<syn::Result<_>>()?
        })
    }
}

impl ToTokens for HasEntrypointsImplItem {
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
                }
            }
        });
    }
}

/// One `odra::contract_def::Entrypoint { .. }`.
struct EntrypointDef {
    name: String,
    /// Argument name and unreferenced type.
    args: Vec<(String, syn::Type)>,
    is_mut: bool,
    return_cl_type: syn::Expr,
    /// `init` and `upgrade` are `EntrypointType::Constructor`, the rest `Public`.
    is_constructor: bool,
    is_payable: bool,
    is_non_reentrant: bool
}

impl EntrypointDef {
    fn try_new(f: &FnIR) -> syn::Result<Self> {
        Ok(Self {
            name: f.name_str(),
            args: f
                .args()
                .iter()
                .map(|arg| (arg.name_str(), utils::ty::unreferenced_ty(arg.ty())))
                .collect(),
            is_mut: f.is_mut(),
            return_cl_type: wasm_parts::return_cl_type(f),
            is_constructor: matches!(f.fn_type(), FnType::Constructor | FnType::Upgrader),
            is_payable: f.is_payable(),
            is_non_reentrant: f.is_non_reentrant()
        })
    }
}

impl ToTokens for EntrypointDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            is_mut,
            return_cl_type,
            ..
        } = self;
        let ty_entrypoint = utils::ty::entry_point_def();
        let ty_string = utils::ty::string();
        let args = self
            .args
            .iter()
            .map(|(name, ty)| quote!(odra::args::odra_argument::<#ty>(#name)));
        let ty = match self.is_constructor {
            true => utils::ty::entry_point_def_ty_constructor(),
            false => utils::ty::entry_point_def_ty_public()
        };
        let attributes = [
            self.is_payable
                .then(utils::ty::entry_point_def_attr_payable),
            self.is_non_reentrant
                .then(utils::ty::entry_point_def_attr_non_reentrant)
        ]
        .into_iter()
        .flatten();

        tokens.extend(quote! {
            #ty_entrypoint {
                name: #ty_string::from(#name),
                args: odra::prelude::vec![#(#args),*],
                is_mutable: #is_mut,
                return_ty: #return_cl_type,
                ty: #ty,
                attributes: odra::prelude::vec![#(#attributes),*]
            }
        });
    }
}

#[cfg(test)]
mod tests;
