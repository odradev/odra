use std::convert::TryFrom;

use proc_macro2::Ident;

use super::impl_item::ImplItem;

pub struct ModuleImpl {
    impl_items: Vec<ImplItem>,
    ident: Ident,
}

impl ModuleImpl {
    pub fn impl_items(&self) -> &[ImplItem] {
        self.impl_items.as_ref()
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|i| matches!(i, ImplItem::Method(_) | ImplItem::Constructor(_)))
            .collect::<Vec<_>>()
    }

    pub fn public_methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|item| match item {
                ImplItem::Constructor(_) => true,
                ImplItem::Method(m) => m.is_public(),
                ImplItem::Other(_) => false,
            })
            .collect::<Vec<_>>()
    }
}

impl TryFrom<syn::ItemImpl> for ModuleImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        let path = match &*item_impl.self_ty {
            syn::Type::Path(path) => path,
            _ => todo!(),
        };
        let contract_ident = path.path.segments.last().unwrap().clone().ident;
        let items = item_impl
            .items
            .into_iter()
            .map(<ImplItem as TryFrom<_>>::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;

        Ok(Self {
            impl_items: items,
            ident: contract_ident,
        })
    }
}

#[cfg(test)]
mod test {
    use super::ModuleImpl;

    #[test]
    fn impl_items_filtering() {
        let item_impl: syn::ItemImpl = syn::parse_quote! {
            impl Contract {
                #[odra(init)]
                pub fn constructor() {}

                pub(crate) fn crate_public_fn() {}

                pub fn public_fn() {}

                fn private_fn() {}
            }
        };
        let module_impl = ModuleImpl::try_from(item_impl).unwrap();

        assert_eq!(module_impl.methods().len(), 4);
        assert_eq!(module_impl.public_methods().len(), 2);
    }
}
