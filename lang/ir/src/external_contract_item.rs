use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

/// Odra external contract trait definition.
pub struct ExternalContractItem {
    pub item_trait: syn::ItemTrait,
    pub item_ref: RefItem
}

impl ExternalContractItem {
    pub fn parse(_attr: TokenStream, item: TokenStream) -> Result<Self, syn::Error> {
        let item_trait = syn::parse2::<syn::ItemTrait>(item)?;

        let ref_ident = format_ident!("{}Ref", &item_trait.ident);

        let item_ref = RefItem { ident: ref_ident };
        Ok(Self {
            item_trait,
            item_ref
        })
    }
}

pub struct RefItem {
    pub ident: Ident
}
