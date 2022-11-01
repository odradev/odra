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
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.event.struct_ident();

        let casper_code = generate_casper_code(self.event);
        let mock_vm_code = generate_mock_vm_code(self.event);

        quote! {
            impl odra::types::event::Event for #struct_ident {
                fn emit(self) {
                    odra::contract_env::emit_event(self);
                }

                fn name(&self) -> String {
                    String::from(stringify!(#struct_ident))
                }
            }

            #casper_code

            #mock_vm_code
        }
    }
}

fn generate_casper_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let name_literal = quote! { stringify!(#struct_ident) };

    let deserialize_fields = fields
        .iter()
        .map(|ident| quote!(let (#ident, bytes) = odra::types::FromBytes::from_vec(bytes.to_vec())?;))
        .collect::<TokenStream>();

    let construct_struct = fields
        .iter()
        .map(|ident| quote! { #ident, })
        .collect::<TokenStream>();

    let mut sum_serialized_lengths = quote! {
        size += #name_literal.serialized_length();
    };
    sum_serialized_lengths.append_all(
        fields
            .iter()
            .map(|ident| quote!(size += self.#ident.serialized_length();)),
    );

    let append_bytes = fields
        .iter()
        .flat_map(|ident| quote!(vec.extend(self.#ident.to_bytes()?);))
        .collect::<TokenStream>();

    let type_check = quote! {
        let (event_name, bytes): (String, _) = odra::types::FromBytes::from_vec(bytes.to_vec())?;
        if &event_name != #name_literal {
            // TODO: Handle error.
            return Err(odra::types::BytesError::Formatting);
        }
    };

    quote! {
        #[cfg(feature = "casper")]
        impl odra::types::FromBytes for #struct_ident {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::types::BytesError> {
                #type_check
                #deserialize_fields
                let value = #struct_ident {
                    #construct_struct
                };
                Ok((value, &[]))
            }
        }

        #[cfg(feature = "casper")]
        impl odra::types::ToBytes for #struct_ident {
            fn to_bytes(&self) -> Result<Vec<u8>, odra::types::BytesError> {
                let mut vec = Vec::with_capacity(self.serialized_length());
                vec.append(&mut #name_literal.to_bytes()?);
                #append_bytes
                Ok(vec)
            }

            fn serialized_length(&self) -> usize {
                let mut size = 0;
                #sum_serialized_lengths
                size
            }
        }

        #[cfg(feature = "casper")]
        impl odra::types::CLTyped for #struct_ident {
            fn cl_type() -> odra::types::CLType {
                odra::types::CLType::Any
            }
        }
    }
}

fn generate_mock_vm_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let fields_serialization = fields
        .iter()
        .map(|ident| quote!(odra::types::BorshSerialize::serialize(&self.#ident, writer)?;))
        .collect::<TokenStream>();

    let fields_deserialization = fields
        .iter()
        .map(|ident| quote!(#ident: odra::types::BorshDeserialize::deserialize(buf)?,))
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "mock-vm")]
        impl odra::types::BorshSerialize for #struct_ident {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                #fields_serialization
                Ok(())
            }
        }

        #[cfg(feature = "mock-vm")]
        impl odra::types::BorshDeserialize for #struct_ident {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                Ok(Self {
                    #fields_deserialization
                })
            }
        }
    }
}
