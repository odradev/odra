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

/// `init` without arguments takes no init args; `upgrade` with arguments takes its own struct.
#[test]
fn test_contract_item_args_follow_each_function() {
    let module = quote::quote! {
        impl Erc20 {
            pub fn init(&mut self) {}
            pub fn upgrade(&mut self, admin: Address) {}
        }
    };
    let module = crate::ir::ModuleImplIR::try_from((&quote::quote!(), &module)).unwrap();
    let item = ContractItem::try_from(&module).unwrap();
    test_utils::assert_contains(&item, quote::quote!(type InitArgs = odra::host::NoArgs;));
    test_utils::assert_contains(&item, quote::quote!(type UpgradeArgs = Erc20UpgradeArgs;));
}
