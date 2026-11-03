use super::*;
use crate::test_utils::{self, mock};

#[test]
fn test_factory_module_item_generation() {
    let ir = mock::factory_module_definition();
    let actual = FactoryModuleItem::try_from(&ir).expect("A valid FactoryModuleItem");

    let expected = quote::quote! {
        #[automatically_derived]
        #[odra::factory]
        pub struct CounterPackFactory;
    };
    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_non_factory_module_item_generation() {
    let ir = mock::module_definition();
    let actual = FactoryModuleItem::try_from(&ir).expect("A valid FactoryModuleItem");

    let expected = quote::quote! {};
    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_factory_module_impl_item_generation() {
    let ir = mock::module_factory_on();
    let actual = FactoryModuleImplItem::try_from(&ir).expect("A valid FactoryModuleImplItem");

    let expected = quote::quote! {
        #[automatically_derived]
        #[odra::factory]
        impl Erc20Factory {
            /// Returns the total supply of the token.
            #[allow(unused_variables)]
            pub fn total_supply(&self) -> U256 {
                self.env().revert(OdraError::ExecutionError(ExecutionError::FactoryModuleCall));
            }
            /// Pay to mint.
            #[odra(payable)]
            #[allow(unused_variables)]
            pub fn pay_to_mint(&mut self) {
                self.env().revert(OdraError::ExecutionError(ExecutionError::FactoryModuleCall));
            }
        }
    };
    test_utils::assert_eq(actual, expected);
}
