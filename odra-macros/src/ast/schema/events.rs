use quote::ToTokens;

use crate::ir::{EnumeratedTypedField, ModuleStructIR};

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

        let events_chain = self.fields
            .iter()
            .map(|f| {
                let ty = &f.ty;
                quote::quote!(.chain(<#ty as odra::schema::SchemaEvents>::schema_events()))
            })
            .collect::<Vec<_>>();

        let types_chain = self.fields
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn custom_types_works() {
        let ir = test_utils::mock::module_definition();
        let item = SchemaEventsItem::try_from(&ir).unwrap();
        let expected = quote::quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for CounterPack {
                fn schema_events() -> odra::prelude::Vec<odra::schema::casper_contract_schema::Event> {
                    odra::prelude::vec::Vec::<odra::schema::casper_contract_schema::Event>::new()
                        .into_iter()
                        .chain(odra::prelude::vec![
                            odra::schema::event(
                                &<OnTransfer as odra::casper_event_standard::EventInstance>::name()
                            ),
                            odra::schema::event(
                                &<OnApprove as odra::casper_event_standard::EventInstance>::name()
                            )
                        ])
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::schema_events())
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::schema_events())
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::schema_events())
                        .chain(<Var<u32> as odra::schema::SchemaEvents>::schema_events())
                        .chain(<Mapping<u8, Counter> as odra::schema::SchemaEvents>::schema_events())
                        .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::Event>>()
                        .into_iter()
                        .collect()
                }

                fn custom_types() -> odra::prelude::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    odra::prelude::vec::Vec::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                        .into_iter()
                        .chain(<OnTransfer as odra::schema::SchemaCustomTypes>::schema_types())
                        .chain(<OnApprove as odra::schema::SchemaCustomTypes>::schema_types())
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                        .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                        .chain(<Var<u32> as odra::schema::SchemaEvents>::custom_types())
                        .chain(<Mapping<u8, Counter> as odra::schema::SchemaEvents>::custom_types())
                        .collect::<odra::prelude::BTreeSet<Option<odra::schema::casper_contract_schema::CustomType>>>()
                        .into_iter()
                        .collect()
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
