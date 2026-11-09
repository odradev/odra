use super::*;
use crate::test_utils;

#[test]
fn storage_layout_works() {
    let ir = test_utils::mock::module_definition();
    let item = SchemaStorageLayoutItem::try_from(&ir).unwrap();
    let expected = quote::quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaStorageLayout for CounterPack {
            fn storage_kind() -> odra::schema::StorageKind {
                odra::schema::StorageKind::Module {
                    fields: odra::prelude::vec![
                        odra::schema::StorageField::new(
                            "counter0",
                            1u8,
                            <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                        ),
                        odra::schema::StorageField::new(
                            "counter1",
                            2u8,
                            <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                        ),
                        odra::schema::StorageField::new(
                            "counter2",
                            3u8,
                            <SubModule<Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                        ),
                        odra::schema::StorageField::new(
                            "counters",
                            4u8,
                            <Var<u32> as odra::schema::SchemaStorageLayout>::storage_kind()
                        ),
                        odra::schema::StorageField::new(
                            "counters_map",
                            5u8,
                            <Mapping<u8, Counter> as odra::schema::SchemaStorageLayout>::storage_kind()
                        )
                    ]
                }
            }

            fn custom_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                odra::prelude::vec::Vec::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                    .into_iter()
                    .chain(<SubModule<Counter> as odra::schema::SchemaStorageLayout>::custom_types())
                    .chain(<SubModule<Counter> as odra::schema::SchemaStorageLayout>::custom_types())
                    .chain(<SubModule<Counter> as odra::schema::SchemaStorageLayout>::custom_types())
                    .chain(<Var<u32> as odra::schema::SchemaStorageLayout>::custom_types())
                    .chain(<Mapping<u8, Counter> as odra::schema::SchemaStorageLayout>::custom_types())
                    .collect::<odra::prelude::BTreeSet<Option<odra::schema::casper_contract_schema::CustomType>>>()
                    .into_iter()
                    .collect()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn explicit_layout_works() {
    let module = quote::quote!(
        pub struct Decimals;
    );
    let attr = quote::quote!(layout = odra::schema::named_key::<u8>("decimals"));
    let ir = ModuleStructIR::try_from((&attr, &module)).unwrap();
    let item = SchemaStorageLayoutItem::try_from(&ir).unwrap();
    let expected = quote::quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaStorageLayout for Decimals {
            fn storage_kind() -> odra::schema::StorageKind {
                odra::schema::named_key::<u8>("decimals")
            }

            fn custom_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                odra::prelude::vec::Vec::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                    .into_iter()
                    .collect::<odra::prelude::BTreeSet<Option<odra::schema::casper_contract_schema::CustomType>>>()
                    .into_iter()
                    .collect()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}
