use crate::test_utils;
use quote::quote;

use super::HasEventsImplItem;

#[test]
fn counter_pack() {
    let module = test_utils::mock::module_definition();
    let expected = quote!(
        impl odra::contract_def::HasEvents for CounterPack {
            fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                let events = odra::prelude::vec![
                    <OnTransfer as odra::contract_def::IntoEvent>::into_event(),
                    <OnApprove as odra::contract_def::IntoEvent>::into_event()
                ];
                odra::prelude::BTreeSet::<odra::contract_def::Event>::new()
                    .into_iter()
                    .chain(events)
                    .chain(<Mapping<u8, Counter> as odra::contract_def::HasEvents>::events())
                    .chain(<SubModule<Counter> as odra::contract_def::HasEvents>::events())
                    .chain(<Var<u32> as odra::contract_def::HasEvents>::events())
                    .collect()
            }

            fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                let __odra_result = odra::prelude::BTreeMap::from_iter(
                    odra::prelude::vec![
                        (<OnTransfer as odra::casper_event_standard::EventInstance>::name(), <OnTransfer as odra::casper_event_standard::EventInstance>::schema()),
                        (<OnApprove as odra::casper_event_standard::EventInstance>::name(), <OnApprove as odra::casper_event_standard::EventInstance>::schema())
                    ]
                );
                __odra_result
                    .into_iter()
                    .chain(<Mapping<u8, Counter> as odra::contract_def::HasEvents>::event_schemas())
                    .chain(<SubModule<Counter> as odra::contract_def::HasEvents>::event_schemas())
                    .chain(<Var<u32> as odra::contract_def::HasEvents>::event_schemas())
                    .collect()
            }
        }
    );
    let actual = HasEventsImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
