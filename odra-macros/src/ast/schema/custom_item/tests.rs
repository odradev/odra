use super::*;
use crate::test_utils;
use quote::quote;

#[test]
fn test_struct() {
    let ir = test_utils::mock::custom_struct();
    let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for MyType {
            fn schema_types(
            ) -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                odra::prelude::BTreeSet::<
                    Option<odra::schema::casper_contract_schema::CustomType>
                >::new()
                .into_iter()
                .chain(odra::prelude::vec![Some(odra::schema::custom_struct(
                    "MyType",
                    odra::prelude::vec![
                        odra::schema::struct_member::<String>("a"),
                        odra::schema::struct_member::<u32>("b"),
                    ]
                ))])
                .chain(odra::prelude::vec![])
                .chain(<String as odra::schema::SchemaCustomTypes>::schema_types())
                .chain(<u32 as odra::schema::SchemaCustomTypes>::schema_types())
                .collect::<odra::prelude::Vec<_>>()
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::NamedCLTyped for MyType {
            fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                odra::schema::casper_contract_schema::NamedCLType::Custom(
                    odra::prelude::String::from("MyType")
                )
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomElement for MyType {}
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_unit_enum() {
    let ir = test_utils::mock::custom_enum();
    let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for MyType {
            fn schema_types(
            ) -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                odra::prelude::BTreeSet::<
                    Option<odra::schema::casper_contract_schema::CustomType>
                >::new()
                .into_iter()
                .chain(odra::prelude::vec![Some(odra::schema::custom_enum(
                    "MyType",
                    odra::prelude::vec![
                        odra::schema::enum_variant("A", 10u16),
                        odra::schema::enum_variant("B", 11u16),
                    ]
                ))])
                .chain(odra::prelude::vec![])
                .collect::<odra::prelude::Vec<_>>()
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::NamedCLTyped for MyType {
            fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                odra::schema::casper_contract_schema::NamedCLType::Custom(
                    odra::prelude::String::from("MyType")
                )
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomElement for MyType {}
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_complex_enum() {
    let ir = test_utils::mock::custom_complex_enum();
    let item = SchemaCustomTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for MyType {
            fn schema_types(
            ) -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                odra::prelude::BTreeSet::<
                    Option<odra::schema::casper_contract_schema::CustomType>
                >::new()
                .into_iter()
                .chain(odra::prelude::vec![Some(odra::schema::custom_enum(
                    "MyType",
                    odra::prelude::vec![
                        odra::schema::enum_custom_type_variant("A", 0u16, "MyType::A"),
                        odra::schema::enum_typed_variant::<(u32, String,)>("B", 1u16),
                        odra::schema::enum_variant("C", 2u16),
                        odra::schema::enum_variant("D", 3u16),
                    ]
                ))])
                .chain(odra::prelude::vec![
                    Some(odra::schema::custom_struct(
                        "MyType::A",
                        odra::prelude::vec![
                            odra::schema::struct_member::<String>("a"),
                            odra::schema::struct_member::<u32>("b")
                        ]
                    )),
                    Some(odra::schema::custom_struct(
                        "MyType::D",
                        odra::prelude::vec![]
                    ))
                ])
                .collect::<odra::prelude::Vec<_>>()
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::NamedCLTyped for MyType {
            fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                odra::schema::casper_contract_schema::NamedCLType::Custom(
                    odra::prelude::String::from("MyType")
                )
            }
        }

        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomElement for MyType {}
    );

    test_utils::assert_eq(item, expected);
}
