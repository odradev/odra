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
                  let (#ident, bytes) = odra::types::FromBytes::from_vec(bytes.to_vec())?;
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
          let (event_name, bytes): (String, _) = odra::types::FromBytes::from_vec(bytes.to_vec())?;
          if &event_name != #name_literal {
            // TODO: Handle error. 
            return Err(odra::types::BytesreprError::Formatting);
          }
        };

        quote! {
            impl odra::types::odra_types::event::Event for #struct_ident {
              fn emit(&self) {
                  odra::contract_env::emit_event(self)
              }

              fn name(&self) -> String {
                  stringify!(#struct_ident).to_string()
              }
            }

            impl odra::types::OdraType for #struct_ident {}

            impl odra::types::FromBytes for #struct_ident {
              fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::types::BytesreprError> {
                #type_check
                #deserialize_fields
                let value = #struct_ident {
                  #construct_struct
                };
                Ok((value, &[]))
              }
            }

          impl odra::types::ToBytes for #struct_ident {

            fn to_bytes(&self) -> Result<Vec<u8>, odra::types::BytesreprError> {
              let mut vec = Vec::with_capacity(self.serialized_length());
              vec.append(&mut #name_literal.to_bytes()?);
              #append_bytes
              Ok(vec)
            }

            fn serialized_length(&self) -> usize {
              let mut size = 0;
              #sum_serialized_lengths
              return size;
            }
          }

          impl odra::types::CLTyped for #struct_ident {
            fn cl_type() -> odra::types::Type {
              odra::types::Type::Any
            }
          }
        }
    }
}
