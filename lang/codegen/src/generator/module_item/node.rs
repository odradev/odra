use derive_more::From;
use odra_ir::module::ModuleStruct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, Token};

use crate::GenerateCode;

#[derive(From)]
pub struct Node<'a> {
    module: &'a ModuleStruct
}

as_ref_for_contract_struct_generator!(Node);

impl GenerateCode for Node<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let struct_ident = &self.module.item.ident;
        let count = self
            .fields_iter()
            .map(|field| &field.ty)
            .map(|ty| quote!(<#ty as odra::types::contract_def::Node>::count()))
            .collect::<Punctuated<TokenStream, Token![+]>>();

        let count = match count.is_empty() {
            true => quote!(0),
            false => quote!(#count)
        };

        let keys = self
            .fields_iter()
            .map(|field| {
                let ty = &field.ty;
                let ident = field.ident.as_ref().unwrap();
                quote! {
                    match <#ty as odra::types::contract_def::Node>::is_leaf() {
                        true => result.push(stringify!(#ident).to_string()),
                        false => result.extend(<#ty as odra::types::contract_def::Node>::keys().iter().map(|k| format!("{}#{}", stringify!(#ident), k)))
                    }
                }
            })
            .collect::<TokenStream>();

        quote! {
            impl odra::types::contract_def::Node for #struct_ident {
                fn count() -> u32 {
                    #count
                }

                fn keys() -> Vec<String> {
                    let mut result = vec![];
                    #keys
                    result
                }

                fn is_leaf() -> bool {
                    false
                }
            }
        }
    }
}

impl<'a> Node<'a> {
    fn fields_iter(&'a self) -> impl Iterator<Item = &Field> + 'a {
        self.module.item.fields.iter()
    }
}
