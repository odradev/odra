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
                let a = field.delegated_fields.iter().map(|f| quote!(#f)).collect::<Punctuated<TokenStream, Token![,]>>();

                // let key = &String::from("shared#value");
                // let aa = key.split("#").take(1).last().unwrap();

                let map = if a.is_empty() {
                    quote!(map(|k| format!("{}#{}", #ident, k)))
                } else {
                    quote!(map(|k: &String| if [#a].contains(&k.split("#").take(1).last().unwrap()) {
                        k.to_owned()
                    } else {
                        format!("{}#{}", #ident, k)
                    }))
                };
                quote! {
                    if <#ty as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from(#ident));
                    } else {
                        result.extend(<#ty as odra::types::contract_def::Node>::_keys()
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

                fn _keys() -> Vec<String> {
                    let mut result = vec![];
                    #keys
                    result
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
