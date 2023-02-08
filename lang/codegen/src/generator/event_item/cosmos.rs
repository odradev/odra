use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let fields_count = event.fields().len();

    let name_literal = quote! { stringify!(#struct_ident) };

    let visitor = format_ident!("{}Visitor", struct_ident);

    let serialize_fields = fields
        .iter()
        .flat_map(|ident| quote!(odra::cosmos::SerializeStruct::serialize_field(&mut _struct, stringify!(#ident), &self.#ident)?;))
        .collect::<TokenStream>();

    let stringified_fields = fields
        .iter()
        .flat_map(|ident| quote!(stringify!(#ident),))
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "cosmos")]
        impl odra::cosmos::serde::Serialize for #struct_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: odra::cosmos::serde::Serializer {
                let mut _struct = serializer.serialize_struct(#name_literal, #fields_count)?;
                #serialize_fields
                odra::cosmos::SerializeStruct::end(_struct)
            }
        }
        #[cfg(feature = "cosmos")]
        struct #visitor;

        #[cfg(feature = "cosmos")]
        impl #visitor {
            fn fields() -> &'static[&'static str] {
                &[#stringified_fields]
            }
        }

        #[cfg(feature = "cosmos")]
        impl<'de> odra::cosmos::serde::de::Visitor<'de> for #visitor {
            type Value = #struct_ident;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an event ")?;
                formatter.write_str(#name_literal)
            }
        }

        #[cfg(feature = "cosmos")]
        impl <'de> odra::cosmos::serde::Deserialize<'de> for #struct_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: odra::cosmos::serde::Deserializer<'de> {
                    deserializer.deserialize_struct(
                        #name_literal,
                        #visitor::fields(),
                        #visitor
                    )
            }
        }
    }
}
