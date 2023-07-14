use derive_more::From;
use odra_ir::module::ModuleStruct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Field, Token};

use crate::GenerateCode;

#[derive(From)]
pub struct NodeItem<'a> {
    module: &'a ModuleStruct
}

as_ref_for_contract_struct_generator!(NodeItem);

impl GenerateCode for NodeItem<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let struct_ident = &self.module.item.ident;
        let count = self
            .fields_iter()
            .map(|field| &field.ty)
            .map(|ty| quote!(<#ty as odra::types::contract_def::Node>::COUNT))
            .collect::<Punctuated<TokenStream, Token![+]>>();

        let count = match count.is_empty() {
            true => quote!(0),
            false => quote!(#count)
        };

        let keys = self
            .module
            .delegated_fields
            .iter()
            .map(|field| {
                let ty = &field.field.ty;
                let ident = field.field.ident.as_ref().unwrap().to_string();
                let fields_collection = field.delegated_fields.iter().map(|f| quote!(#f)).collect::<Punctuated<TokenStream, Token![,]>>();

                let map = if fields_collection.is_empty() {
                    quote!(map(|k| odra::utils::create_key(#ident, k)))
                } else {
                    quote!(map(|k: &String| if [#fields_collection].contains(&k.split(odra::utils::KEY_DELIMITER).take(1).last().unwrap()) {
                        k.to_owned()
                    } else {
                        odra::utils::create_key(#ident, k)
                    }))
                };
                quote! {
                    if <#ty as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from(#ident));
                    } else {
                        result.extend(<#ty as odra::types::contract_def::Node>::__keys()
                            .iter()
                            .#map)
                    }
                }
            })
            .collect::<TokenStream>();

        quote! {
            impl odra::types::contract_def::Node for #struct_ident {
                const IS_LEAF: bool = false;
                const COUNT: u32 = #count;

                fn __keys() -> Vec<String> {
                    let mut result = vec![];
                    #keys
                    result
                }
            }
        }
    }
}

impl<'a> NodeItem<'a> {
    fn fields_iter(&'a self) -> impl Iterator<Item = &Field> + 'a {
        self.module.item.fields.iter()
    }
}
