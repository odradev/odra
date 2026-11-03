use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    ast::{
        deployer_item::DeployerItem,
        factory::{FactoryDeployImplItem, FactoryHostRefItem},
        host_ref_item::{HasIdentTraitImplItem, HostRefItem},
        parts_utils::{UsePreludeItem, UseSuperItem}
    },
    ir::ModuleImplIR,
    utils
};

/// Which contract the test parts are built for: the module itself, or the factory that deploys
/// instances of it. They differ in the host ref and the deployer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestPartsKind {
    Module,
    Factory
}

/// Emits `pub use __xxx_test_parts::*;` outside wasm, so the host ref and the deployer are in
/// scope next to the module.
pub struct TestPartsReexportItem {
    mod_ident: Ident
}

impl TryFrom<&ModuleImplIR> for TestPartsReexportItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            mod_ident: module.test_parts_mod_ident()?
        })
    }
}

impl ToTokens for TestPartsReexportItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mod_ident = &self.mod_ident;
        let not_wasm32_attr = utils::attr::not_wasm32();
        tokens.extend(quote! {
            #not_wasm32_attr
            pub use #mod_ident::*;
        });
    }
}

/// Emits `mod __xxx_test_parts`, compiled outside wasm only: the host ref, its `HasIdent` impl
/// and the deployer, the pieces a test uses to deploy and call the contract.
pub struct TestPartsItem {
    mod_ident: Ident,
    host_ref: HostRef,
    has_ident_impl: HasIdentTraitImplItem,
    deployer: Deployer
}

impl TestPartsItem {
    pub(crate) fn try_new(module: &ModuleImplIR, kind: TestPartsKind) -> syn::Result<Self> {
        let (host_ref, deployer) = match kind {
            TestPartsKind::Module => (
                HostRef::Module(HostRefItem::try_from(module)?),
                Deployer::Module(DeployerItem::try_from(module)?)
            ),
            TestPartsKind::Factory => (
                HostRef::Factory(FactoryHostRefItem::try_from(module)?),
                Deployer::Factory(FactoryDeployImplItem::try_from(module)?)
            )
        };
        Ok(Self {
            mod_ident: module.test_parts_mod_ident()?,
            host_ref,
            has_ident_impl: HasIdentTraitImplItem::try_from(module)?,
            deployer
        })
    }
}

impl TryFrom<&ModuleImplIR> for TestPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        Self::try_new(module, TestPartsKind::Module)
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
    let not_wasm32_attr = utils::attr::not_wasm32();
    let use_super = UseSuperItem;
    let use_prelude = UsePreludeItem;
    quote! {
        #not_wasm32_attr
        mod #ident {
            #use_super
            #use_prelude
            #body
        }
    }
}

enum HostRef {
    Module(HostRefItem),
    Factory(FactoryHostRefItem)
}

impl ToTokens for HostRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            HostRef::Module(item) => item.to_tokens(tokens),
            HostRef::Factory(item) => item.to_tokens(tokens)
        }
    }
}

enum Deployer {
    Module(DeployerItem),
    Factory(FactoryDeployImplItem)
}

impl ToTokens for Deployer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Deployer::Module(item) => item.to_tokens(tokens),
            Deployer::Factory(item) => item.to_tokens(tokens)
        }
    }
}

#[cfg(test)]
mod tests;
