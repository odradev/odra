use super::HasIdentImplItem;
use crate::test_utils;
use quote::quote;

#[test]
fn test_entrypoints() {
    let module = test_utils::mock::module_definition();
    let expected = quote!(
        impl odra::contract_def::HasIdent for CounterPack {
            fn ident() -> odra::prelude::string::String {
                odra::prelude::string::String::from("CounterPack")
            }
            fn contract_name() -> odra::prelude::string::String {
                odra::prelude::string::String::from("MyCounterPack")
            }
        }
    );
    let actual = HasIdentImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}

#[test]
fn no_name_attribute_no_name_fn() {
    let module = test_utils::mock::factory_module_definition();
    let expected = quote!(
        impl odra::contract_def::HasIdent for CounterPack {
            fn ident() -> odra::prelude::string::String {
                odra::prelude::string::String::from("CounterPack")
            }
        }
    );
    let actual = HasIdentImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
