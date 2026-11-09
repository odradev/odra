use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{
    ast::contract_ref_item::{ContractRefKind, RefItem},
    ir::ModuleImplIR
};

/// Emits the contract ref of a factory: the module's contract ref built for the factory kind,
/// see [RefItem].
pub struct FactoryRefItem(RefItem);

impl TryFrom<&ModuleImplIR> for FactoryRefItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        RefItem::try_new(module, ContractRefKind::Factory).map(Self)
    }
}

impl ToTokens for FactoryRefItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests;
