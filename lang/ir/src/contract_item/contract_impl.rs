use std::convert::TryFrom;

use proc_macro2::Ident;

use super::impl_item::ImplItem;

pub struct ContractImpl {
    impl_items: Vec<ImplItem>,
    ident: Ident,
}

impl ContractImpl {
    pub fn impl_items(&self) -> &[ImplItem] {
        self.impl_items.as_ref()
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|i| match i {
                ImplItem::Method(_) => true,
                ImplItem::Constructor(_) => true,
                _ => false,
            })
            .collect::<Vec<_>>()
    }
}

impl TryFrom<syn::ItemImpl> for ContractImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        let path = match &*item_impl.self_ty {
            syn::Type::Path(path) => path,
            _ => todo!(),
        };
        let contract_ident = path.path.segments.last().unwrap().clone().ident;
        let items = item_impl
            .items
            .clone()
            .into_iter()
            .map(|item| <ImplItem as TryFrom<_>>::try_from(item))
            .collect::<Result<Vec<_>, syn::Error>>()?;

        Ok(Self {
            impl_items: items,
            ident: contract_ident,
        })
    }
}
