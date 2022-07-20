use std::convert::TryFrom;

use proc_macro2::TokenStream;

use self::{module_impl::ModuleImpl, module_struct::ModuleStruct};

pub mod constructor;
pub mod impl_item;
pub mod method;
pub mod module_impl;
pub mod module_struct;

pub enum ModuleItem {
    Struct(ModuleStruct),
    Impl(ModuleImpl),
}

impl ModuleItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_struct = syn::parse2::<syn::ItemStruct>(item.clone());
        let item_impl = syn::parse2::<syn::ItemImpl>(item.clone());

        if let Ok(item) = item_struct {
            return Ok(ModuleItem::Struct(ModuleStruct::from(item)));
        }

        if let Ok(item) = item_impl {
            let item = ModuleImpl::try_from(item)?;
            return Ok(ModuleItem::Impl(item));
        }

        Err(syn::Error::new_spanned(
            item,
            "ContractItem is neither a struct nor an impl block.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn invalid_usage() {
        let result = ModuleItem::parse(
            quote!(),
            quote!(
                fn some_fn(x: u32) -> u32 {
                    x + 1
                }
            ),
        );
        assert!(result.is_err());

        let result = ModuleItem::parse(
            quote!(),
            quote!(
                enum A {}
            ),
        );
        assert!(result.is_err());
    }

    #[test]
    fn struct_block() {
        let result = ModuleItem::parse(
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
        let result = ModuleItem::parse(
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
