use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{
    ast::deployer_item::{DeployerItem, DeployerKind},
    ir::ModuleImplIR
};

/// Emits the deployer of a factory: the module's deployer built for the factory kind, see
/// [DeployerItem].
pub struct FactoryDeployImplItem(DeployerItem);

impl TryFrom<&ModuleImplIR> for FactoryDeployImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        DeployerItem::try_new(module, DeployerKind::Factory).map(Self)
    }
}

impl ToTokens for FactoryDeployImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}
