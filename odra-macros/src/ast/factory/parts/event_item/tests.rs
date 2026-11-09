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
                let mut __odra_result = odra::prelude::BTreeMap::new();
                __odra_result.insert(
                    <CounterPackContractDeployed as odra::casper_event_standard::EventInstance>::name(),
                    <CounterPackContractDeployed as odra::casper_event_standard::EventInstance>::schema()
                );
                __odra_result
            }
        }
    );
    test_utils::assert_eq(item, expected);
}
