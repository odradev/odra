use derive_more::From;
use odra_ir::InstanceItem as IrInstanceItem;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::GenerateCode;

#[derive(From)]
pub struct InstanceItem<'a> {
    item: &'a IrInstanceItem
}

impl GenerateCode for InstanceItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let item_struct = self.item.data_struct();

        let ident = &self.item.ident();

        let fields: TokenStream = item_struct
            .clone()
            .fields
            .into_iter()
            .flat_map(|field| WrappedField(field).to_token_stream())
            .collect();

        quote! {
            impl odra::Instance for #ident {
                fn instance(namespace: &str) -> Self {
                    Self {
                        #fields
                    }
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
            #ident: odra::Instance::instance(
                &[stringify!(#ident), namespace]
                    .iter()
                    .filter_map(|str| match str.is_empty() {
                        true => None,
                        false => Some(*str),
                    })
                    .collect::<Vec<_>>()
                    .join("_")
            ),
        });
    }
}
