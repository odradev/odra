use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{
        deployer_item::DeployerItem,
        host_ref_item::{HasIdentTraitImplItem, HostRefItem},
        parts_utils::{UsePreludeItem, UseSuperItem},
        Target
    },
    ir::ModuleImplIR
};

/// Emits `pub use __xxx_test_parts::*;` outside wasm, so the host ref and the deployer are in
/// scope next to the module.
pub struct TestPartsReexportItem {
    mod_ident: Ident
}

impl TryFrom<&ModuleImplIR> for TestPartsReexportItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            mod_ident: module.test_parts_mod_ident()
        })
    }
}

impl ToTokens for TestPartsReexportItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mod_ident = &self.mod_ident;
        tokens.extend(quote! {
            #[cfg(not(target_arch = "wasm32"))]
            pub use #mod_ident::*;
        });
    }
}

/// Emits `mod __xxx_test_parts`, compiled outside wasm only: the host ref, its `HasIdent` impl
/// and the deployer, the pieces a test uses to deploy and call the contract.
pub struct TestPartsItem {
    mod_ident: Ident,
    host_ref: HostRefItem,
    has_ident_impl: HasIdentTraitImplItem,
    deployer: DeployerItem
}

impl TestPartsItem {
    pub(crate) fn try_new(module: &ModuleImplIR, target: Target) -> syn::Result<Self> {
        Ok(Self {
            mod_ident: module.test_parts_mod_ident(),
            host_ref: HostRefItem::try_new(module, target)?,
            has_ident_impl: HasIdentTraitImplItem::try_from(module)?,
            deployer: DeployerItem::try_new(module, target)?
        })
    }
}

impl TryFrom<&ModuleImplIR> for TestPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, Target::Module)
    }
}

impl ToTokens for TestPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            mod_ident,
            host_ref,
            has_ident_impl,
            deployer
        } = self;
        tokens.extend(not_wasm32_mod(
            mod_ident,
            quote! {
                #host_ref
                #has_ident_impl
                #deployer
            }
        ));
    }
}

/// `#[cfg(not(target_arch = "wasm32"))] mod #ident { use super::*; use odra::prelude::*; #body }`,
/// the shape of every test parts module.
pub(super) fn not_wasm32_mod(ident: &Ident, body: TokenStream) -> TokenStream {
    let use_super = UseSuperItem;
    let use_prelude = UsePreludeItem;
    quote! {
        #[cfg(not(target_arch = "wasm32"))]
        mod #ident {
            #use_super
            #use_prelude
            #body
        }
    }
}

#[cfg(test)]
mod tests;
