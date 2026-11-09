use super::RefHelpersItem;
use crate::test_utils;

#[test]
fn ref_helpers() {
    let item = quote::quote! {
        /// Helpers.
        impl crate::token::Token {
            /// Balance of the owner.
            pub fn owner_balance(&self) -> U256 {
                self.balance_of(self.owner())
            }
        }
    };
    let item = RefHelpersItem::try_from(&item).unwrap();
    let expected = quote::quote! {
        const _: () = {
            use odra::ContractAddress as _;
            /// Helpers.
            impl crate::token::TokenContractRef {
                /// Balance of the owner.
                pub fn owner_balance(&self) -> U256 {
                    self.balance_of(self.owner())
                }
            }
        };

        #[cfg(not(target_arch = "wasm32"))]
        const _: () = {
            use odra::host::HostRef as _;
            /// Helpers.
            impl crate::token::TokenHostRef {
                /// Balance of the owner.
                pub fn owner_balance(&self) -> U256 {
                    self.balance_of(self.owner())
                }
            }
        };
    };
    test_utils::assert_eq(item, expected);
}

#[test]
fn rejects_trait_impl() {
    let item = quote::quote!(impl Foo for Token {});
    assert!(RefHelpersItem::try_from(&item).is_err());
}

#[test]
fn rejects_generics() {
    let item = quote::quote!(impl<T> Token<T> {});
    assert!(RefHelpersItem::try_from(&item).is_err());
    let item = quote::quote!(impl Token<u32> {});
    assert!(RefHelpersItem::try_from(&item).is_err());
}

#[test]
fn rejects_non_impl() {
    let item = quote::quote!(
        struct Token;
    );
    assert!(RefHelpersItem::try_from(&item).is_err());
}
