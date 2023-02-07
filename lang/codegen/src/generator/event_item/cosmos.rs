use odra_ir::EventItem as IrEventItem;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let fields_count = event.fields().len();

    let name_literal = quote! { stringify!(#struct_ident) };

    let visitor = format_ident!("{}Visitor", struct_ident);

    let serialize_fields = fields
        .iter()
        .flat_map(|ident| quote!(serde::ser::SerializeStruct::serialize_field(&mut _struct, stringify!(#ident), &self.#ident)?;))
        .collect::<TokenStream>();

    let stringified_fields = fields
        .iter()
        .flat_map(|ident| quote!(stringify!(#ident),))
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "cosmos")]
        impl serde::Serialize for #struct_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer {
                let mut _struct = serializer.serialize_struct(#name_literal, #fields_count)?;
                #serialize_fields
                serde::ser::SerializeStruct::end(_struct)
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
        impl<'de> serde::de::Visitor<'de> for #visitor {
            type Value = #struct_ident;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an event ")?;
                formatter.write_str(#name_literal)
            }
        }
        
        #[cfg(feature = "cosmos")]
        impl <'de> serde::Deserialize<'de> for #struct_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de> {
                    deserializer.deserialize_struct(
                        #name_literal, 
                        #visitor::fields(), 
                        #visitor
                    )
            }
        }
    }
}
