
use crate::ir::{errors, ModuleStructIR, TypeIR};
use crate::utils;

#[derive(syn_derive::ToTokens)]
pub struct ImplItem {
    impl_token: syn::Token![impl],
    ty: syn::Type,
    for_token: syn::Token![for],
    for_ty: syn::Type
}

impl ImplItem {
    pub fn from<T: Named>(named: &T, for_ty: &syn::Type) -> syn::Result<Self> {
        let ty_from = utils::ty::from(&named.name()?);
        Ok(Self {
            impl_token: Default::default(),
            ty: ty_from,
            for_token: Default::default(),
            for_ty: for_ty.clone()
        })
    }
}

pub trait Named {
    fn name(&self) -> syn::Result<syn::Ident>;
}

impl Named for TypeIR {
    fn name(&self) -> syn::Result<syn::Ident> {
        match self.self_code() {
            syn::Item::Struct(i) => Ok(i.ident.clone()),
            syn::Item::Enum(i) => Ok(i.ident.clone()),
            _ => Err(errors::type_on_other_item(self.self_code()))
        }
    }
}

impl Named for ModuleStructIR {
    fn name(&self) -> syn::Result<syn::Ident> {
        Ok(self.module_ident())
    }
}
