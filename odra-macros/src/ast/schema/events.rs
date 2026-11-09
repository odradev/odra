use quote::{format_ident, ToTokens};

use crate::ir::{EnumeratedTypedField, ModuleStructIR};

/// Emits `impl odra::schema::SchemaEvents for Xxx` for a module: the events of
/// `#[odra::module(events = [..])]` and of every field, each once, and their custom types.
pub struct SchemaEventsItem {
    module_ident: syn::Ident,
    events: Vec<syn::Type>,
    fields: Vec<EnumeratedTypedField>
}

impl ToTokens for SchemaEventsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let events = self.events.iter().map(|ty| {
            quote::quote!(odra::schema::event(&<#ty as odra::casper_event_standard::EventInstance>::name()))
        }).collect::<Vec<_>>();

        let types = self.events.iter().map(|event| {
            quote::quote!(.chain(<#event as odra::schema::SchemaCustomTypes>::schema_types()))
        }).collect::<Vec<_>>();

        let events_chain = self
            .fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::schema_events()))
            })
            .collect::<Vec<_>>();

        let types_chain = self
            .fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::custom_types()))
            })
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for #module_ident {
                fn schema_events() -> odra::prelude::Vec<odra::schema::casper_contract_schema::Event> {
                    odra::prelude::vec::Vec::<odra::schema::casper_contract_schema::Event>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![#(#events),*])
                        #(#events_chain)*
                        .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::Event>>()
                        .into_iter()
                        .collect()
                }

                fn custom_types() -> odra::prelude::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::vec::Vec::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        #(#types)*
                        #(#types_chain)*
                        .collect::<odra::prelude::BTreeSet<Option<odra::schema::casper_contract_schema::CustomType>>>()
                        .into_iter()
                        .collect()
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleStructIR> for SchemaEventsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
            events: ir.events(),
            fields: ir.typed_fields()?
        })
    }
}

/// Emits `impl odra::schema::SchemaEvents` for a factory: its one `XxxContractDeployed` event.
pub struct FactorySchemaEventsItem {
    module_ident: syn::Ident,
    event_ident: syn::Ident,
}

impl TryFrom<&ModuleStructIR> for FactorySchemaEventsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident(),
            event_ident: format_ident!("{}ContractDeployed", ir.module_ident()),
        })
    }
}

impl ToTokens for FactorySchemaEventsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let event_ident = &self.event_ident;
        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for #module_ident {
                fn schema_events() -> odra::prelude::Vec<odra::schema::casper_contract_schema::Event> {
                    odra::prelude::vec![
                        odra::schema::event(
                            &<#event_ident as odra::casper_event_standard::EventInstance>::name()
                        )
                    ]
                }

                fn custom_types() -> odra::prelude::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    <#event_ident as odra::schema::SchemaCustomTypes>::schema_types()
                }
            }
        };

        item.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests;
