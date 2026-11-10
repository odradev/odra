use super::*;
use crate::test_utils;

#[test]
fn test_contract_item() {
    let module = test_utils::mock::module_factory_impl();

    let item = FactoryContractItem::try_from(&module).expect("Failed to create ContractItem");
    let expected = quote::quote! {
        impl odra::OdraContract for Erc20Factory {
            #[cfg(not(target_arch = "wasm32"))]
            type HostRef = Erc20FactoryHostRef;

            type ContractRef = Erc20FactoryContractRef;

            #[cfg(not(target_arch = "wasm32"))]
            type InitArgs = odra::host::NoArgs;

            #[cfg(not(target_arch = "wasm32"))]
            type UpgradeArgs = odra::host::NoArgs;

            #[cfg(not(target_arch = "wasm32"))]
            fn contract_events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                <Erc20Factory as odra::contract_def::HasEvents>::events()
                    .into_iter()
                    .chain(<Erc20 as odra::contract_def::HasEvents>::events())
                    .collect()
            }
        }
    };
    test_utils::assert_eq(item, expected);
}
