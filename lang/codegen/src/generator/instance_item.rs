use derive_more::From;
use odra_ir::InstanceItem as IrInstanceItem;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, Token};

use crate::GenerateCode;

#[derive(From)]
pub struct InstanceItem<'a> {
    item: &'a IrInstanceItem
}

impl GenerateCode for InstanceItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let ident = &self.item.ident();

        let static_fields: TokenStream = self
            .fields_iter()
            .flat_map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote!(let (#ident, keys) = odra::StaticInstance::instance(&keys);)
            })
            .collect();

        let init = self
            .fields_iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote!(#ident)
            })
            .collect::<Punctuated<TokenStream, Token![,]>>();

        let dynamic_fields = self
            .fields_iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote!(#ident: odra::DynamicInstance::instance(&[namespace, stringify!(#ident).as_bytes()].concat()))
            })
            .collect::<Punctuated<TokenStream, Token![,]>>();

        quote! {
            impl odra::StaticInstance for #ident {
                fn instance<'a>(keys: &'a [&'a str]) -> (Self, &'a [&'a str]) {
                    #static_fields
                    (
                        Self { #init },
                        keys
                    )
                }
            }

            impl odra::DynamicInstance for #ident {
                fn instance(namespace: &[u8]) -> Self {
                    Self {
                        #dynamic_fields
                    }
                }
            }
        }
    }
}

impl<'a> InstanceItem<'a> {
    fn fields_iter(&'a self) -> impl Iterator<Item = &'a Field> + 'a {
        self.item.data_struct().fields.iter()
    }
}
