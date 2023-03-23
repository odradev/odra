use std::convert::TryFrom;

use proc_macro2::Ident;

use super::{
    delegate::Delegate,
    impl_item::{Entrypoint, ImplItem}
};

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
    is_trait_implementation: bool
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
            .filter(|i| {
                matches!(
                    i,
                    ImplItem::Method(_)
                        | ImplItem::Constructor(_)
                        | ImplItem::DelegationStatement(_)
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn public_methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|item| match item {
                ImplItem::Constructor(_) => true,
                ImplItem::Method(m) => self.is_trait_implementation || m.is_public(),
                ImplItem::Other(_) => false,
                ImplItem::DelegationStatement(_) => true
            })
            .collect::<Vec<_>>()
    }

    pub fn is_trait_implementation(&self) -> bool {
        self.is_trait_implementation
    }
}

impl TryFrom<syn::ItemImpl> for ModuleImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        let is_trait_implementation = item_impl.trait_.is_some();
        let path = match &*item_impl.self_ty {
            syn::Type::Path(path) => path,
            _ => todo!()
        };
        let contract_ident = path.path.segments.last().unwrap().clone().ident;

        let delegation_stmts = item_impl
            .items
            .clone()
            .into_iter()
            .filter_map(|item| match item {
                syn::ImplItem::Macro(macro_item) => Some(macro_item),
                _ => None
            })
            .map(|macro_item| syn::parse2::<Delegate>(macro_item.mac.tokens))
            .collect::<Result<Vec<_>, syn::Error>>()?;

        let delegation_stmts = delegation_stmts
            .into_iter()
            .flat_map(|d| d.stmts)
            .map(ImplItem::DelegationStatement)
            .collect::<Vec<_>>();

        let mut items = item_impl
            .items
            .into_iter()
            .filter(|item| matches!(item, syn::ImplItem::Method(_)))
            .map(<ImplItem as TryFrom<_>>::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;

        items.extend(delegation_stmts);

        Ok(Self {
            impl_items: items,
            ident: contract_ident,
            is_trait_implementation
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

                delegate! {
                    to self.a {
                        pub fn public_fn_del(&self);
                        pub fn private_fn_del(&self);
                    }
                }
            }
        };
        let module_impl = ModuleImpl::try_from(item_impl).unwrap();

        assert_eq!(module_impl.methods().len(), 4);
        assert_eq!(module_impl.public_methods().len(), 2);
    }
}
