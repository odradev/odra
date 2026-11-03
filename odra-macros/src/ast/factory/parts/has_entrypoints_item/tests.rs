use crate::test_utils;
use quote::quote;

use super::FactoryHasEntrypointsImplItem;

#[test]
fn test_entrypoints() {
    let module = test_utils::mock::module_factory_impl();
    let expected = quote!(
        impl odra::contract_def::HasEntrypoints for Erc20Factory {
            fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                odra::prelude::vec![
                    odra::entry_point::EntryPoint::Constructor {
                        args: odra::prelude::vec![]
                    },
                    odra::entry_point::EntryPoint::Factory {
                       args: vec![
                            odra::args::parameter::<odra::prelude::string::String>("contract_name"),
                            odra::args::parameter::<u32>("value")
                        ],
                    },
                    odra::entry_point::EntryPoint::FactoryUpgrade {
                       args: vec![
                            odra::args::parameter::<odra::prelude::string::String>("contract_name")
                        ],
                    },
                    odra::entry_point::EntryPoint::FactoryBatchUpgrade
                ].into_iter().map(TryInto::try_into).collect::<Result<_, _>>().unwrap_or_default()
            }
        }
    );
    let actual = FactoryHasEntrypointsImplItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
