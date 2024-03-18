use quote::ToTokens;

use crate::ir::ModuleStructIR;

pub struct SchemaEventsItem {
    module_ident: syn::Ident,
    events: Vec<syn::Type>
}

impl ToTokens for SchemaEventsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let events = self.events.iter().map(|event| {
            quote::quote! {
                odra::schema::event(&<#event as odra::casper_event_standard::EventInstance>::name())
            }
        }).collect::<Vec<_>>();

        let item = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for #module_ident {
                fn schema_events() -> Vec<odra::schema::casper_contract_schema::Event> {
                    vec![ #(#events),* ]
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
            events: ir.events()
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
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for CounterPack {
                fn schema_events() -> Vec<odra::schema::casper_contract_schema::Event> {
                    vec![
                        odra::schema::event(
                            &<OnTransfer as odra::casper_event_standard::EventInstance>::name()
                        ),
                        odra::schema::event(
                            &<OnApprove as odra::casper_event_standard::EventInstance>::name()
                        )
                    ]
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
