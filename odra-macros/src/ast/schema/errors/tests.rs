use super::*;
use crate::test_utils;

#[test]
fn custom_types_works() {
    let ir = test_utils::mock::module_definition();
    let item = SchemaErrorsItem::try_from(&ir).unwrap();
    let expected = quote::quote!(
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaErrors for CounterPack {
            fn schema_errors(
            ) -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError>
            {
                odra::prelude::BTreeSet::<odra::schema::casper_contract_schema::UserError>::new()
                    .into_iter()
                    .chain(<Erc20Errors as odra::schema::SchemaErrors>::schema_errors())
                    .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                    .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                    .chain(<SubModule<Counter> as odra::schema::SchemaErrors>::schema_errors())
                    .chain(<Var<u32> as odra::schema::SchemaErrors>::schema_errors())
                    .chain(<Mapping<u8, Counter> as odra::schema::SchemaErrors>::schema_errors())
                    .collect::<odra::prelude::BTreeSet<odra::schema::casper_contract_schema::UserError>>()
                    .into_iter()
                    .collect::<odra::prelude::Vec<_>>()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_odra_error_item() {
    let ty = test_utils::mock::custom_enum();
    let item = SchemaErrorItem::new(ty.ident(), ty.error_variants().unwrap());
    let expected = quote::quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaErrors for MyType {
            fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                odra::prelude::vec![
                    odra::schema::error("A", "Description of A", 10u16),
                    odra::schema::error("B", "Description of B", 11u16),
                ]
            }
        }
    };
    test_utils::assert_eq(item, expected);
}

#[test]
fn test_odra_error_item_with_multiline_docs() {
    let ty = test_utils::mock::custom_enum_with_multiline_docs();
    let item = SchemaErrorItem::new(ty.ident(), ty.error_variants().unwrap());
    let expected = quote::quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaErrors for MyType {
            fn schema_errors() -> odra::prelude::Vec<odra::schema::casper_contract_schema::UserError> {
                odra::prelude::vec![
                    odra::schema::error("A", "Description of A. More details about A.", 10u16),
                    odra::schema::error("B", "Description of B", 11u16),
                ]
            }
        }
    };
    test_utils::assert_eq(item, expected);
}
