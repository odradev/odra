use proc_macro2::TokenStream;
use syn::{ImplItemMethod, ItemImpl};

use self::{contract_struct::ContractStruct, contract_impl::ContractImpl};

pub mod contract_impl;
pub mod contract_struct;

pub struct ContractItem {
    contract_struct: Option<ContractStruct>,
    contract_impl: Option<ContractImpl>
}

impl ContractItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_struct = syn::parse2::<syn::ItemStruct>(item.clone());
        let item_impl = syn::parse2::<syn::ItemImpl>(item.clone());

        if item_struct.is_err() && item_impl.is_err() {
            return Err(syn::Error::new_spanned(
                item,
                "ContractItem is neither a struct nor an impl block.",
            ));
        }

        Ok(Self {
            contract_struct: item_struct.and_then(|item| Ok(ContractStruct::from(item))).ok(),
            contract_impl: item_impl.and_then(|item| Ok(ContractImpl::from(item))).ok(),
        })
    }

    pub fn contract_struct(&self) -> Option<&ContractStruct> {
        self.contract_struct.as_ref()
    }

    pub fn contract_impl(&self) -> Option<&ContractImpl> {
        self.contract_impl.as_ref()
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
