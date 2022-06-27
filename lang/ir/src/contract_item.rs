use std::convert::TryFrom;

use proc_macro2::TokenStream;
use syn::{ImplItemMethod, ItemImpl};

use self::{contract_impl::ContractImpl, contract_struct::ContractStruct};

pub mod constructor;
pub mod contract_impl;
pub mod contract_struct;
pub mod impl_item;
pub mod method;

pub enum ContractItem {
    Struct(ContractStruct),
    Impl(ContractImpl),
}

impl ContractItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_struct = syn::parse2::<syn::ItemStruct>(item.clone());
        let item_impl = syn::parse2::<syn::ItemImpl>(item.clone());

        if item_struct.is_ok() {
            let item = item_struct.unwrap();
            return Ok(ContractItem::Struct(ContractStruct::from(item)));
        }

        if item_impl.is_ok() {
            let item = item_impl.unwrap();
            let item = ContractImpl::try_from(item)?;
            return Ok(ContractItem::Impl(item));
        }

        Err(syn::Error::new_spanned(
            item,
            "ContractItem is neither a struct nor an impl block.",
        ))
    }
}

pub fn extract_methods<'a>(item: ItemImpl) -> Vec<ImplItemMethod> {
    item.items
        .into_iter()
        .filter_map(|item| match item {
            syn::ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn invalid_usage() {
        let result = ContractItem::parse(
            quote!(),
            quote!(
                fn some_fn(x: u32) -> u32 {
                    x + 1
                }
            ),
        );
        assert!(result.is_err());

        let result = ContractItem::parse(
            quote!(),
            quote!(
                enum A {}
            ),
        );
        assert!(result.is_err());
    }

    #[test]
    fn struct_block() {
        let result = ContractItem::parse(
            quote!(),
            quote!(
                struct ContractItem {
                    x: u32,
                    name: String,
                }
            ),
        );
        assert!(result.is_ok())
    }

    #[test]
    fn impl_block() {
        let result = ContractItem::parse(
            quote!(),
            quote!(
                impl ContractItem {
                    fn a() {}
                }
            ),
        );
        assert!(result.is_ok())
    }
}
