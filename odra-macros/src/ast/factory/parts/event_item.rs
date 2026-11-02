

use quote::{format_ident, ToTokens, TokenStreamExt};

use crate::{ast::{events_item::EventsFnsItem, utils::ImplItem}, utils::{self, misc::AsType}, ModuleStructIR};

pub struct FactoryEventItem {
    event_ident: syn::Ident,
}

impl TryFrom<&'_ ModuleStructIR> for FactoryEventItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            event_ident: format_ident!("{}ContractDeployed", module.module_ident()),
        })
    }
}

impl ToTokens for FactoryEventItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let event_ident = &self.event_ident;
        let ty_string = utils::ty::string();
        let ty_address = utils::ty::address();
        tokens.append_all(quote::quote! {
            #[odra::event]
            pub struct #event_ident {
                pub contract_name: #ty_string,
                pub contract_address: #ty_address
            }
        });
    }
}

pub struct FactoryHasEventsImplItem {
    impl_item: ImplItem,
    event_ident: EventsFnsItem
}

impl TryFrom<&'_ ModuleStructIR> for FactoryHasEventsImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let event_ident = format_ident!("{}ContractDeployed", ir.module_ident());
        Ok(Self {
            impl_item: ImplItem::has_events(ir)?,
            event_ident: EventsFnsItem::single(&event_ident.as_type())
        })
    }
}
impl ToTokens for FactoryHasEventsImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let impl_item = &self.impl_item;
        let event_ident = &self.event_ident;
        tokens.append_all(quote::quote! {
            #impl_item {
                #event_ident
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_contract_item() {
        let module = test_utils::mock::factory_module_definition();

        let item = FactoryHasEventsImplItem::try_from(&module).expect("Failed to create ContractItem");
        let expected = quote::quote!(
            impl odra::contract_def::HasEvents for CounterPack {
                fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                    odra::prelude::vec![
                        <CounterPackContractDeployed as odra::contract_def::IntoEvent>::into_event()
                    ]
                }

                fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                    odra::prelude::BTreeMap::from_iter(
                        odra::prelude::vec![(
                            <CounterPackContractDeployed as odra::casper_event_standard::EventInstance>::name(), 
                            <CounterPackContractDeployed as odra::casper_event_standard::EventInstance>::schema()
                        )]
                    )
                }
            }
        );
        test_utils::assert_eq(item, expected);
    }
}
