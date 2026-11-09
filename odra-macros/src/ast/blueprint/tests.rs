use super::BlueprintItem;
use crate::test_utils;
use quote::quote;

#[test]
fn blueprint_item() {
    let module = test_utils::mock::module_impl();
    let item = BlueprintItem::try_from(&module).unwrap();
    let expected = quote!(
        #[cfg(any(odra_module = "Erc20", odra_module = "unknown_crate::Erc20"))]
        mod __erc20_schema {
            use super::*;

            #[no_mangle]
            #[cfg(not(target_arch = "wasm32"))]
            fn module_schema() -> odra::contract_def::ContractBlueprint {
                odra::contract_def::ContractBlueprint::new::<Erc20>()
            }
        }
    );
    test_utils::assert_eq(item, &expected);
}
