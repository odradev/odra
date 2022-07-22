use odra_ir::InstanceItem;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn generate_code(item: InstanceItem) -> TokenStream {
    let item_struct = item.data_struct();

    let ident = &item.ident();

    let fields: TokenStream = item_struct
        .clone()
        .fields
        .into_iter()
        .flat_map(|field| WrappedField(field).to_token_stream())
        .collect();

    quote! {
        impl odra::instance::Instance for #ident {
            fn instance(namespace: &str) -> Self {
                Self {
                    #fields
                 }
            }
        }
    }
}

struct WrappedField(pub syn::Field);

impl ToTokens for WrappedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.0.ident.as_ref().unwrap();
        tokens.extend(quote! {
            #ident: odra::instance::Instance::instance(
                [stringify!(#ident), namespace]
                    .iter()
                    .filter_map(|str| match str.is_empty() {
                        true => None,
                        false => Some(str.to_string()),
                    })
                    .collect::<Vec<String>>()
                    .join("_")
                    .as_str()
            ),
        });
    }
}
