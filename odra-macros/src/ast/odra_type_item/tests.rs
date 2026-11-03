use super::*;
use crate::test_utils;
use quote::quote;

#[test]
fn test_struct() {
    let ir = test_utils::mock::custom_struct();
    let item = OdraTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[derive(Clone, PartialEq, Eq, Debug)]
        struct MyType {
            a: String,
            b: u32,
        }

        impl odra::casper_types::bytesrepr::FromBytes for MyType {
            fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                let (a, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                let (b, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;

                Ok((Self {
                    a,
                    b
                }, __odra_bytes))
            }
        }

        impl odra::casper_types::bytesrepr::ToBytes for MyType {
            fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                let mut __odra_result = odra::prelude::vec::Vec::with_capacity(self.serialized_length());
                __odra_result.extend(odra::casper_types::bytesrepr::ToBytes::to_bytes(&self.a)?);
                __odra_result.extend(odra::casper_types::bytesrepr::ToBytes::to_bytes(&self.b)?);
                Ok(__odra_result)
            }

            fn serialized_length(&self) -> usize {
                let mut __odra_result = 0;
                __odra_result += self.a.serialized_length();
                __odra_result += self.b.serialized_length();
                __odra_result
            }
        }

        impl odra::casper_types::CLTyped for MyType {
            fn cl_type() -> odra::casper_types::CLType {
                odra::casper_types::CLType::Any
            }
        }

        impl odra::contract_def::HasEvents for MyType {
            fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                odra::prelude::vec::Vec::new()
            }

            fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                odra::prelude::BTreeMap::new()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_unit_enum() {
    let ir = test_utils::mock::custom_enum();
    let item = OdraTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[derive(Clone, PartialEq, Eq, Debug)]
        enum MyType {
            /// Description of A
            A = 10,
            /// Description of B
            B
        }

        impl odra::casper_types::bytesrepr::FromBytes for MyType {
            fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                let (__odra_result, __odra_bytes): (u8, _) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                match __odra_result {
                    x if x == Self::A as u8 => Ok((Self::A, __odra_bytes)),
                    x if x == Self::B as u8 => Ok((Self::B, __odra_bytes)),
                    _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
                }
            }
        }

        impl odra::casper_types::bytesrepr::ToBytes for MyType {
            fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                Ok(odra::prelude::vec![self.clone() as u8])
            }

            fn serialized_length(&self) -> usize {
                odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH
            }
        }

        impl odra::casper_types::CLTyped for MyType {
            fn cl_type() -> odra::casper_types::CLType {
                odra::casper_types::CLType::U8
            }
        }

        impl odra::contract_def::HasEvents for MyType {
            fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                odra::prelude::vec::Vec::new()
            }

            fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                odra::prelude::BTreeMap::new()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_complex_enum() {
    let ir = test_utils::mock::custom_complex_enum();
    let item = OdraTypeItem::try_from(&ir).unwrap();
    let expected = quote!(
        #[derive(Clone, PartialEq, Eq, Debug)]
        enum MyType {
            /// Description of A
            A { a: String, b: u32 },
            /// Description of B
            B(u32, String),
            /// Description of C
            C(),
            /// Description of D
            D {},
        }

        impl odra::casper_types::bytesrepr::FromBytes for MyType {
            fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper_types::bytesrepr::Error> {
                let (__odra_result, __odra_bytes): (u8, _) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                match __odra_result {
                    0u8 => {
                        let (a, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                        let (b, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                        Ok((Self::A { a, b }, __odra_bytes))
                    },
                    1u8 => {
                        let (f0, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                        let (f1, __odra_bytes) = odra::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                        Ok((Self::B(f0, f1), __odra_bytes))
                    },
                    2u8 => Ok((Self::C(), __odra_bytes)),
                    3u8 => Ok((Self::D {}, __odra_bytes)),
                    _ => Err(odra::casper_types::bytesrepr::Error::Formatting),
                }
            }
        }

        impl odra::casper_types::bytesrepr::ToBytes for MyType {
            fn to_bytes(&self) -> Result<odra::prelude::vec::Vec<u8>, odra::casper_types::bytesrepr::Error> {
                match self {
                    Self::A { a, b } => {
                        let mut __odra_result = odra::prelude::vec![0u8];
                        __odra_result.extend_from_slice(&a.to_bytes()?);
                        __odra_result.extend_from_slice(&b.to_bytes()?);
                        Ok(__odra_result)
                    },
                    Self::B(f0, f1) => {
                        let mut __odra_result = odra::prelude::vec![1u8];
                        __odra_result.extend_from_slice(&f0.to_bytes()?);
                        __odra_result.extend_from_slice(&f1.to_bytes()?);
                        Ok(__odra_result)
                    },
                    Self::C() => {
                        let mut __odra_result = odra::prelude::vec![2u8];
                        Ok(__odra_result)
                    },
                    Self::D {} => {
                        let mut __odra_result = odra::prelude::vec![3u8];
                        Ok(__odra_result)
                    }
                }
            }

            fn serialized_length(&self) -> usize {
                match self {
                    Self::A { a, b } => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH + a.serialized_length() + b.serialized_length(),
                    Self::B(f0, f1) => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH + f0.serialized_length() + f1.serialized_length(),
                    Self::C() => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH,
                    Self::D {} => odra::casper_types::bytesrepr::U8_SERIALIZED_LENGTH,
                }
            }
        }

        impl odra::casper_types::CLTyped for MyType {
            fn cl_type() -> odra::casper_types::CLType {
                odra::casper_types::CLType::Any
            }
        }

        impl odra::contract_def::HasEvents for MyType {
            fn events() -> odra::prelude::vec::Vec<odra::contract_def::Event> {
                odra::prelude::vec::Vec::new()
            }

            fn event_schemas() -> odra::prelude::BTreeMap<odra::prelude::string::String, odra::casper_event_standard::Schema> {
                odra::prelude::BTreeMap::new()
            }
        }
    );

    test_utils::assert_eq(item, expected);
}

#[test]
fn test_union() {
    let ir = test_utils::mock::custom_union();
    let item = OdraTypeItem::try_from(&ir);
    assert!(item.is_err());
}
