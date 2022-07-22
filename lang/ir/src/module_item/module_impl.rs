use std::convert::TryFrom;

use proc_macro2::Ident;

use super::impl_item::ImplItem;

/// Odra module implementation block.
///
/// # Examples
/// ```
/// # <odra_ir::module::ModuleImpl as TryFrom<syn::ItemImpl>>::try_from(syn::parse_quote! {
/// impl MyModule {
///     #[odra(init)]
///     #[other_attribute]
///     pub fn set_initial_value(&self, value: u32) {
///         // initialization logic goes here
///     }
///
///     pub fn set_value(&self, value: u32) {
///         // logic goes here
///     }
/// }
/// # }).unwrap();
/// ```
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
