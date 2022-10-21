use derive_more::From;
use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

use crate::GenerateCode;

#[derive(From)]
pub struct EventItem<'a> {
    event: &'a IrEventItem,
}

impl GenerateCode for EventItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let struct_ident = self.event.struct_ident();
        let fields = self.event.fields();

        let name_literal = quote! { stringify!(#struct_ident) };

        let deserialize_fields = fields
            .iter()
            .map(|ident| {
                quote! {
                  let (#ident, bytes) = odra::types::FromBytes::from_vec(bytes)?;
                }
            })
            .collect::<TokenStream>();

        let construct_struct = fields
            .iter()
            .map(|ident| quote! { #ident, })
            .collect::<TokenStream>();

        let mut sum_serialized_lengths = quote! {
          size += #name_literal.serialized_length();
        };
        sum_serialized_lengths.append_all(fields.iter().map(|ident| {
            quote! {
              size += self.#ident.serialized_length();
            }
        }));

        let append_bytes = fields
            .iter()
            .flat_map(|ident| {
                quote! {
                  vec.extend(self.#ident.to_bytes()?);
                }
            })
            .collect::<TokenStream>();

        let type_check = quote! {
          let (event_name, bytes): (String, _) = odra::types::FromBytes::from_vec(bytes)?;
          if &event_name != #name_literal {
            return core::result::Result::Err(odra::types::event::EventError::UnexpectedType(event_name));
          }
        };

        quote! {
            impl odra::types::event::Event for #struct_ident {
              fn emit(&self) {
                  odra::contract_env::emit_event(self)
              }

              fn name(&self) -> String {
                  stringify!(#struct_ident).to_string()
              }
            }

            impl odra::types::ToBytes for #struct_ident {
              type Error = odra::types::event::EventError;

              fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
                  core::result::Result::Ok(<Self as odra::types::ToBytes>::to_bytes(self)?)
              }
            }

            impl odra::types::FromBytes for #struct_ident {
              type Error = odra::types::event::EventError;

              type Item = Self;

              fn deserialize(bytes: Vec<u8>) -> Result<(Self::Item, Vec<u8>), Self::Error> {
                #type_check
                #deserialize_fields
                let value = #struct_ident {
                  #construct_struct
                };
                Ok((value, bytes))
              }
            }

          impl odra::types::ToBytes for #struct_ident {
            fn serialized_length(&self) -> usize {
              let mut size = 0;
              #sum_serialized_lengths
              return size;
            }

            fn to_bytes(&self) -> Result<Vec<u8>, odra::types::bytesrepr::Error> {
              let mut vec = Vec::with_capacity(self.serialized_length());
              vec.append(&mut #name_literal.to_bytes()?);
              #append_bytes
              Ok(vec)
            }
          }
        }
    }
}
