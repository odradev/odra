use odra_ir::{EventItem as IrEventItem, Field};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let fields_serialization = fields
        .iter()
        .map(|Field { ident, ..}| quote!(odra::types::BorshSerialize::serialize(&self.#ident, writer)?;))
        .collect::<TokenStream>();

    let fields_deserialization = fields
        .iter()
        .map(
            |Field { ident, .. }| quote!(#ident: odra::types::BorshDeserialize::deserialize(buf)?,)
        )
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "mock-vm")]
        impl odra::types::BorshSerialize for #struct_ident {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                odra::types::BorshSerialize::serialize(stringify!(#struct_ident), writer)?;

                #fields_serialization
                Ok(())
            }
        }

        #[cfg(feature = "mock-vm")]
        impl odra::types::BorshDeserialize for #struct_ident {

            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                let _ = <String as odra::types::BorshDeserialize>::deserialize(buf)?;
                Ok(Self {
                    #fields_deserialization
                })
            }
        }
    }
}
