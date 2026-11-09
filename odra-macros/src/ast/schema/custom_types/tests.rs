use super::*;
use crate::test_utils;

#[test]
fn custom_types_works() {
    let ir = test_utils::mock::module_impl();
    let item = SchemaCustomTypesItem::try_from(&ir).unwrap();
    let expected = quote::quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for Erc20 {
            fn schema_types(
            ) -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>>
            {
                odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                    .into_iter()
                    .chain(<Option<U256> as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<U256 as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Address as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Maybe<String> as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<odra::prelude::vec::Vec<Address> as odra::schema::SchemaCustomTypes>::schema_types())
                    .chain(<Self as odra::schema::SchemaEvents>::custom_types())
                    .collect()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}
