use odra_ir::{EventItem as IrEventItem, Field};
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let name_literal = quote! { stringify!(#struct_ident) };

    let deserialize_fields = fields
        .iter()
        .map(|Field { ident, ..}| quote!(let (#ident, bytes) = odra::casper::casper_types::bytesrepr::FromBytes::from_vec(bytes.to_vec())?;))
        .collect::<TokenStream>();

    let construct_struct = fields
        .iter()
        .map(|Field { ident, .. }| quote! { #ident, })
        .collect::<TokenStream>();

    let mut sum_serialized_lengths = quote! {
        size += #name_literal.serialized_length();
    };
    sum_serialized_lengths.append_all(
        fields
            .iter()
            .map(|Field { ident, .. }| quote!(size += self.#ident.serialized_length();))
    );

    let append_bytes = fields
        .iter()
        .flat_map(|Field { ident, .. }| quote!(vec.extend(self.#ident.to_bytes()?);))
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "casper")]
        impl odra::casper::casper_types::bytesrepr::FromBytes for #struct_ident {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra::casper::casper_types::bytesrepr::Error> {
                let (_, bytes): (String, Vec<u8>) = odra::casper::casper_types::bytesrepr::FromBytes::from_vec(bytes.to_vec())?;
                #deserialize_fields
                let value = #struct_ident {
                    #construct_struct
                };
                Ok((value, &[]))
            }
        }

        #[cfg(feature = "casper")]
        impl odra::casper::casper_types::bytesrepr::ToBytes for #struct_ident {
            fn to_bytes(&self) -> Result<Vec<u8>, odra::casper::casper_types::bytesrepr::Error> {
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
        impl odra::casper::casper_types::CLTyped for #struct_ident {
            fn cl_type() -> odra::casper::casper_types::CLType {
                odra::casper::casper_types::CLType::Any
            }
        }
    }
}
