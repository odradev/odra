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
            fn schema_events() -> odra::prelude::Vec<odra::schema::casper_contract_schema::Event>
            {
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

            fn custom_types(
            ) -> odra::prelude::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                odra::prelude::vec::Vec::<
                    Option<odra::schema::casper_contract_schema::CustomType>
                >::new()
                .into_iter()
                .chain(<OnTransfer as odra::schema::SchemaCustomTypes>::schema_types())
                .chain(<OnApprove as odra::schema::SchemaCustomTypes>::schema_types())
                .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                .chain(<SubModule<Counter> as odra::schema::SchemaEvents>::custom_types())
                .chain(<Var<u32> as odra::schema::SchemaEvents>::custom_types())
                .chain(<Mapping<u8, Counter> as odra::schema::SchemaEvents>::custom_types())
                .collect::<odra::prelude::BTreeSet<
                    Option<odra::schema::casper_contract_schema::CustomType>
                >>()
                .into_iter()
                .collect()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_factory_module() {
    let module = test_utils::mock::factory_module_definition();
    let item = FactorySchemaEventsItem::try_from(&module).unwrap();
    let expected = quote::quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaEvents for CounterPack {
            fn schema_events() -> odra::prelude::Vec<odra::schema::casper_contract_schema::Event>
            {
                odra::prelude::vec![
                    odra::schema::event(
                        &<CounterPackContractDeployed as odra::casper_event_standard::EventInstance>::name()
                    )
                ]
            }

            fn custom_types(
                ) -> odra::prelude::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                <CounterPackContractDeployed as odra::schema::SchemaCustomTypes>::schema_types()
            }
        }
    };
    test_utils::assert_eq(item, expected);
}
