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

        tokens.extend(quote! {
            impl odra::contract_def::HasEntrypoints for #module_ident {
                fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
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
        let args = self
            .args
            .iter()
            .map(|(name, ty)| quote!(odra::args::odra_argument::<#ty>(#name)));
        let ty = match self.is_constructor {
            true => quote!(odra::contract_def::EntrypointType::Constructor),
            false => quote!(odra::contract_def::EntrypointType::Public)
        };
        let attributes = [
            self.is_payable
                .then(|| quote!(odra::contract_def::EntrypointAttribute::Payable)),
            self.is_non_reentrant
                .then(|| quote!(odra::contract_def::EntrypointAttribute::NonReentrant))
        ]
        .into_iter()
        .flatten();

        tokens.extend(quote! {
            odra::contract_def::Entrypoint {
                name: odra::prelude::string::String::from(#name),
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
