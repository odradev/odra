use derive_more::From;
use odra_ir::module::ModuleStruct;
use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{punctuated::Punctuated, Token};

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
            .map(|ident| quote!(self.#ident.count()))
            .collect::<Punctuated<TokenStream, Token![+]>>();

        let count = match count.is_empty() {
            true => quote!(0),
            false => quote!(#count),
        };

        let ranges = self.fields_iter()
            .map(|ident| quote! {
                let start = ranges.last().map(|r: &core::ops::Range<u32>| r.end).unwrap_or_default();
                let end = start + self.#ident.count();

                ranges.push(start..end);
            })
            .collect::<TokenStream>();

        let keys = self
            .fields_iter()
            .map(|ident| quote! {
                match self.#ident.is_leaf() {
                    true => result.push(stringify!(#ident).to_string()),
                    false => result.extend(self.#ident.keys().iter().map(|k| format!("{}_{}", stringify!(#ident), k)))
                }
            })
            .collect::<TokenStream>();

        quote! {
            impl odra::Node for #struct_ident {
                fn count(&self) -> u32 {
                    #count
                }

                fn keys(&self) -> Vec<String> {
                    let mut result = Vec::with_capacity(self.count() as usize);
                    #keys
                    result
                }

                fn is_leaf(&self) -> bool {
                    false
                }

                fn key_ranges(&self) -> Vec<core::ops::Range<u32>> {
                    let mut ranges = Vec::with_capacity(self.count() as usize);
                    #ranges
                    ranges
                }
            
                fn range(&self) -> core::ops::Range<u32> {
                    0..self.count()
                }
            }
        }
    }


}


impl<'a> Node<'a> {
    fn fields_iter(&'a self) -> impl Iterator<Item = Ident> + 'a {
        self.module
            .item
            .fields
            .iter()
            .filter_map(|f| match &f.ident {
                Some(i) => Some(i.clone()),
                None => None,
            })
    }
}