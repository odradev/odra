use crate::ir::{ModuleStructIR, TypeIR};
use crate::utils;
use crate::utils::misc::AsType;

#[derive(syn_derive::ToTokens)]
pub struct ImplItem {
    impl_token: syn::Token![impl],
    ty: syn::Type,
    for_token: syn::Token![for],
    for_ty: syn::Type
}

impl ImplItem {
    fn new<T: Named>(named: &T, ty: syn::Type) -> syn::Result<Self> {
        Ok(Self {
            impl_token: Default::default(),
            ty,
            for_token: Default::default(),
            for_ty: named.name()?.as_type()
        })
    }

    pub fn from_bytes(ir: &TypeIR) -> syn::Result<Self> {
        Self::new(ir, utils::ty::from_bytes())
    }

    pub fn to_bytes(ir: &TypeIR) -> syn::Result<Self> {
        Self::new(ir, utils::ty::to_bytes())
    }

    pub fn cl_typed(ir: &TypeIR) -> syn::Result<Self> {
        Self::new(ir, utils::ty::cl_typed())
    }

    pub fn clone<T: Named>(named: &T) -> syn::Result<Self> {
        Self::new(named, utils::ty::clone())
    }

    pub fn has_events<T: Named>(named: &T) -> syn::Result<Self> {
        Self::new(named, utils::ty::has_events())
    }

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
        Ok(self.self_code().ident.clone())
    }
}

impl Named for ModuleStructIR {
    fn name(&self) -> syn::Result<syn::Ident> {
        Ok(self.module_ident())
    }
}
