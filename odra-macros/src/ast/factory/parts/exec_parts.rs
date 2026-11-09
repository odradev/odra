use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::{
    ast::exec_parts::{ExecPartsKind, ModuleExecPartsItem},
    ir::ModuleImplIR
};

/// Emits the exec parts of a factory: the module's exec parts built for the factory kind, see
/// [ModuleExecPartsItem].
pub struct FactoryExecPartsItem(ModuleExecPartsItem);

impl TryFrom<&ModuleImplIR> for FactoryExecPartsItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleImplIR) -> Result<Self, Self::Error> {
        ModuleExecPartsItem::try_new(module, ExecPartsKind::Factory).map(Self)
    }
}

impl ToTokens for FactoryExecPartsItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests;
