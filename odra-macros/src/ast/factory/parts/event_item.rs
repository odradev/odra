use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::{ast::events_item::HasEventsImplItem, ir::ModuleStructIR};

/// Emits `#[odra::event] pub struct XxxContractDeployed { contract_name, contract_address }`,
/// the event a factory emits for every child it deploys or upgrades.
pub struct FactoryEventItem {
    event_ident: Ident
}

impl TryFrom<&ModuleStructIR> for FactoryEventItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            event_ident: deployed_event_ident(module)
        })
    }
}

impl ToTokens for FactoryEventItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event_ident = &self.event_ident;
        tokens.extend(quote! {
            #[odra::event]
            pub struct #event_ident {
                pub contract_name: odra::prelude::string::String,
                pub contract_address: odra::prelude::Address
            }
        });
    }
}

/// Emits `impl HasEvents` for the factory: its one `XxxContractDeployed` event, see
/// [HasEventsImplItem].
pub struct FactoryHasEventsImplItem(HasEventsImplItem);

impl TryFrom<&ModuleStructIR> for FactoryHasEventsImplItem {
    type Error = syn::Error;

    fn try_from(module: &ModuleStructIR) -> Result<Self, Self::Error> {
        let event_ident = deployed_event_ident(module);
        Ok(Self(HasEventsImplItem::only(
            module.module_ident(),
            syn::parse_quote!(#event_ident)
        )))
    }
}

impl ToTokens for FactoryHasEventsImplItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

fn deployed_event_ident(module: &ModuleStructIR) -> Ident {
    format_ident!("{}ContractDeployed", module.module_ident())
}

#[cfg(test)]
mod tests;
