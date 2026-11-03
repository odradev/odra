use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::ir::ModuleImplIR;

/// Emits `impl odra::OdraContract for Xxx`, naming the host ref, the contract ref and the
/// argument types of `init` and `upgrade` (`NoArgs` when the function takes none or is absent).
pub struct ContractItem {
    module_ident: Ident,
    host_ref_ident: Ident,
    contract_ref_ident: Ident,
    init_args: syn::Path,
    upgrade_args: syn::Path
}

impl TryFrom<&ModuleImplIR> for ContractItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        let no_args: syn::Path = syn::parse_quote!(odra::host::NoArgs);
        let has_args = |f: Option<crate::ir::FnIR>| f.is_some_and(|f| f.has_args());
        Ok(Self {
            module_ident: module.module_ident()?,
            host_ref_ident: module.host_ref_ident()?,
            contract_ref_ident: module.contract_ref_ident()?,
            init_args: match has_args(module.constructor()) {
                true => module.init_args_ident()?.into(),
                false => no_args.clone()
            },
            upgrade_args: match has_args(module.upgrader()) {
                true => module.upgrade_args_ident()?.into(),
                false => no_args
            }
        })
    }
}

impl ToTokens for ContractItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            module_ident,
            host_ref_ident,
            contract_ref_ident,
            init_args,
            upgrade_args
        } = self;

        tokens.extend(quote! {
            impl odra::OdraContract for #module_ident {
                #[cfg(not(target_arch = "wasm32"))]
                type HostRef = #host_ref_ident;

                type ContractRef = #contract_ref_ident;

                #[cfg(not(target_arch = "wasm32"))]
                type InitArgs = #init_args;

                #[cfg(not(target_arch = "wasm32"))]
                type UpgradeArgs = #upgrade_args;
            }
        });
    }
}

#[cfg(test)]
mod tests;
