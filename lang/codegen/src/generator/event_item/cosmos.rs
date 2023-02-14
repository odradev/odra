use odra_ir::{EventItem as IrEventItem, Field};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_code(event: &IrEventItem) -> TokenStream {
    let struct_ident = event.struct_ident();
    let fields = event.fields();

    let add_attrs = fields
        .iter()
        .flat_map(|Field { ident, is_optional }| {
            if *is_optional {
                quote! {
                    let ev = if let Some(#ident) = self.#ident {
                        ev.add_attribute(stringify!(#ident), #ident.to_string())
                    } else {
                        ev.add_attribute(stringify!(#ident), "null")
                    };
                }
            } else {
                quote!(let ev = ev.add_attribute(stringify!(#ident), self.#ident.to_string());)
            }
        })
        .collect::<TokenStream>();

    let parse_event_fields = fields
        .iter()
        .flat_map(|Field { ident, is_optional }| {
            let parse_arg = if *is_optional {
                quote!(let #ident = #ident.value.parse().ok();)
            } else {
                quote!(let #ident = #ident.value.parse().expect("invalid arg");)
            };

            quote! {
                let #ident = value.attributes.iter()
                    .find(|attr| attr.key == stringify!(#ident))
                    .expect("missing argument");
                #parse_arg
            }
        })
        .collect::<TokenStream>();

    let event_fields = fields
        .iter()
        .flat_map(|Field { ident, .. }| quote!(#ident,))
        .collect::<TokenStream>();

    quote! {
        #[cfg(feature = "cosmos")]
        impl Into<odra::cosmos::CosmosEvent> for #struct_ident {
            fn into(self) -> odra::cosmos::CosmosEvent {
                let ev = odra::cosmos::CosmosEvent::new(<#struct_ident as odra::types::event::OdraEvent>::name());
                #add_attrs
                ev
            }
        }

        #[cfg(feature = "cosmos")]
        impl From<odra::cosmos::CosmosEvent> for #struct_ident {
            fn from(value: odra::cosmos::CosmosEvent) -> Self {
                #parse_event_fields
        
                Self { 
                    #event_fields
                }
            }
        }
    }
}

#[cfg(test)]
mod t {
    use syn::parse_quote;

    #[test]
    fn a() {
        let t: syn::Type = parse_quote!(Option<u32>);
        // dbg!(t);

        match t {
            syn::Type::Path(p) => {
                dbg!(p.path.segments.first().unwrap().ident.to_string());
            }
            _ => todo!()
        }

        assert!(false);
    }
}
