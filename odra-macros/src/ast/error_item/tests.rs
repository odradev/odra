use super::*;
use crate::test_utils;

#[test]
fn test_odra_error_item() {
    let ty = test_utils::mock::custom_enum();
    let item = OdraErrorItem::try_from(&ty).unwrap();
    let expected = quote::quote! {
        enum MyType {
            /// Description of A
            A = 10,
            /// Description of B
            B,
        }

        #[cfg(target_arch = "wasm32")]
        #[automatically_derived]
        impl ::core::convert::From<MyType> for OdraError {
            fn from(error: MyType) -> Self {
                OdraError::user(error as u16)
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[automatically_derived]
        impl ::core::convert::From<MyType> for OdraError {
            fn from(error: MyType) -> Self {
                match error {
                    MyType::A => OdraError::user(error as u16, stringify!(A)),
                    MyType::B => OdraError::user(error as u16, stringify!(B)),
                }
            }
        }
    };
    test_utils::assert_eq(item, expected);
}
