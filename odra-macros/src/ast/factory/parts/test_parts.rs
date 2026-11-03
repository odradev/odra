use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{
    ast::test_parts::{TestPartsItem, TestPartsKind},
    ir::ModuleImplIR
};

/// Emits the test parts of a factory: the module's test parts built for the factory kind, see
/// [TestPartsItem].
pub struct FactoryTestPartsItem(TestPartsItem);

impl TryFrom<&ModuleImplIR> for FactoryTestPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        TestPartsItem::try_new(module, TestPartsKind::Factory).map(Self)
    }
}

impl ToTokens for FactoryTestPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests;
