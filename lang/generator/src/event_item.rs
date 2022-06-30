use odra_ir::event_item::EventItem;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

pub fn generate_code(item: EventItem) -> TokenStream {
    let struct_ident = item.struct_ident();
    let fields = item.fields();

    let name_literal = quote! { stringify!(#struct_ident) };

    let deserialize_fields = fields
        .iter()
        .map(|ident| {
            quote! {
              let (#ident, bytes) = odra::types::bytesrepr::FromBytes::from_bytes(bytes)?;
            }
        })
        .flatten()
        .collect::<TokenStream>();

    let construct_struct = fields
        .iter()
        .map(|ident| quote! { #ident, })
        .flatten()
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
        .map(|ident| {
            quote! {
              vec.extend(self.#ident.to_bytes()?);
            }
        })
        .flatten()
        .collect::<TokenStream>();

    let type_check = quote! {
      let (event_name, bytes): (String, _) = odra::types::bytesrepr::FromBytes::from_bytes(bytes)?;
      if &event_name != #name_literal {
          return core::result::Result::Err(odra::types::bytesrepr::Error::Formatting)
      }
    };

    quote! {
        impl odra::Event for #struct_ident {
            fn emit(&self) {
                odra::env::ContractEnv::emit_event(&<Self as odra::types::bytesrepr::ToBytes>::to_bytes(&self).unwrap())
            }

            fn name(&self) -> &str {
                stringify!(#struct_ident)
            }
        }

        impl odra::types::bytesrepr::FromBytes for #struct_ident {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::types::bytesrepr::Error> {
                #type_check
                #deserialize_fields
                let value = #struct_ident {
                  #construct_struct
                };
                Ok((value, bytes))
            }
        }

      impl odra::types::bytesrepr::ToBytes for #struct_ident {
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
