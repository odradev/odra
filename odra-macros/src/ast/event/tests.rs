use super::OdraEventItem;
use crate::test_utils;
use quote::quote;

#[test]
fn event() {
    let item = OdraEventItem::try_from(&quote!(
        pub struct Transfer {
            pub bytes: U256,
            pub to: Address
        }
    ))
    .unwrap();
    let expected = quote! {
        #[derive(PartialEq, Eq, Debug)]
        pub struct Transfer {
            pub bytes: U256,
            pub to: Address
        }

        impl casper_event_standard::casper_types::CLTyped for Transfer {
            fn cl_type() -> casper_event_standard::casper_types::CLType {
                casper_event_standard::casper_types::CLType::Any
            }
        }

        impl casper_event_standard::casper_types::bytesrepr::ToBytes for Transfer {
            fn to_bytes(&self) -> Result<casper_event_standard::alloc::vec::Vec<u8>, casper_event_standard::casper_types::bytesrepr::Error> {
                use casper_event_standard::casper_types::bytesrepr::ToBytes as _;
                let mut __odra_vec = casper_event_standard::alloc::vec::Vec::with_capacity(self.serialized_length());
                __odra_vec.append(&mut "event_Transfer".to_bytes()?);
                casper_event_standard::validate_type(&self.bytes)?;
                __odra_vec.extend(self.bytes.to_bytes()?);
                casper_event_standard::validate_type(&self.to)?;
                __odra_vec.extend(self.to.to_bytes()?);
                Ok(__odra_vec)
            }

            fn serialized_length(&self) -> usize {
                use casper_event_standard::casper_types::bytesrepr::ToBytes as _;
                let mut __odra_size = 0;
                __odra_size += "event_Transfer".serialized_length();
                __odra_size += self.bytes.serialized_length();
                __odra_size += self.to.serialized_length();
                __odra_size
            }
        }

        impl casper_event_standard::casper_types::bytesrepr::FromBytes for Transfer {
            fn from_bytes(__odra_bytes: &[u8]) -> Result<(Self, &[u8]), casper_event_standard::casper_types::bytesrepr::Error> {
                let (_, __odra_bytes): (casper_event_standard::alloc::string::String, &[u8]) =
                    casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                let (bytes, __odra_bytes) =
                    casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                let (to, __odra_bytes) =
                    casper_event_standard::casper_types::bytesrepr::FromBytes::from_bytes(__odra_bytes)?;
                Ok((Transfer { bytes, to, }, __odra_bytes))
            }
        }

        impl casper_event_standard::EventInstance for Transfer {
            fn name() -> casper_event_standard::alloc::string::String {
                casper_event_standard::alloc::string::String::from("Transfer")
            }

            fn schema() -> casper_event_standard::Schema {
                let mut __odra_schema = casper_event_standard::Schema::new();
                __odra_schema.with_elem("bytes", <U256 as casper_event_standard::casper_types::CLTyped>::cl_type());
                __odra_schema.with_elem("to", <Address as casper_event_standard::casper_types::CLTyped>::cl_type());
                __odra_schema
            }
        }

        impl Transfer {
            #[doc = "Creates a new instance of the Transfer event."]
            pub fn new(bytes: U256, to: Address) -> Self {
                Self { bytes, to }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::NamedCLTyped for Transfer {
            fn ty() -> odra::schema::casper_contract_schema::NamedCLType {
                odra::schema::casper_contract_schema::NamedCLType::Custom(odra::prelude::String::from("Transfer"))
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl odra::schema::SchemaCustomTypes for Transfer {
            fn schema_types() -> odra::prelude::vec::Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                odra::prelude::BTreeSet::<Option<odra::schema::casper_contract_schema::CustomType>>::new()
                    .into_iter()
                    .chain(odra::prelude::vec![Some(odra::schema::custom_struct("Transfer", odra::prelude::vec![
                        odra::schema::struct_member::<U256>("bytes"),
                        odra::schema::struct_member::<Address>("to"),
                    ]))])
                    .chain(<U256 as odra::schema::SchemaEvents>::custom_types())
                    .chain(<Address as odra::schema::SchemaEvents>::custom_types())
                    .collect()
            }
        }
    };
    test_utils::assert_eq(item, expected);
}
