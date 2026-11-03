use super::*;
use crate::test_utils;

#[test]
fn test_contract_item() {
    let module = test_utils::mock::module_impl();

    let item = ContractItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        impl odra::OdraContract for Erc20 {
            #[cfg(not(target_arch = "wasm32"))]
            type HostRef = Erc20HostRef;

            type ContractRef = Erc20ContractRef;

            #[cfg(not(target_arch = "wasm32"))]
            type InitArgs = Erc20InitArgs;

            #[cfg(not(target_arch = "wasm32"))]
            type UpgradeArgs = Erc20UpgradeArgs;
        }
    };
    test_utils::assert_eq(item, expected);
}
