use crate::test_utils;
use quote::quote;

use super::SchemaItem;

#[test]
fn contract_schema_item() {
    let module = test_utils::mock::module_definition();
    let item = SchemaItem::try_from(&module).unwrap();
    let expected = quote!(
        #[automatically_derived]
        #[cfg(all(
            not(target_arch = "wasm32"),
            any(odra_module = "CounterPack", odra_module = "unknown_crate::CounterPack")
        ))]
        mod __counter_pack_contract_schema {
            use super::*;

            #[no_mangle]
            fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema
            {
                let version = match "0.1.0" {
                    "" => env!("CARGO_PKG_VERSION"),
                    _ => "0.1.0"
                };
                let authors = env!("CARGO_PKG_AUTHORS")
                    .to_string()
                    .split(":")
                    .filter_map(|s| if s.is_empty() { None } else { Some(s.trim().to_owned()) })
                    .collect();
                let repository = env!("CARGO_PKG_REPOSITORY");
                let homepage = env!("CARGO_PKG_HOMEPAGE");
                odra::schema::schema::<CounterPack>(
                    "CounterPack",
                    "MyCounterPack",
                    version,
                    authors,
                    repository,
                    homepage
                )
            }
        }
    );
    test_utils::assert_eq(item, &expected);
}
