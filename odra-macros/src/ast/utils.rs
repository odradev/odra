use crate::ir::{StructIR, TypeIR};
use crate::utils;

#[derive(syn_derive::ToTokens)]
pub struct ImplItem {
    impl_token: syn::Token![impl],
    ty: syn::Type,
    for_token: syn::Token![for],
    ident: syn::Ident
}

impl ImplItem {
    fn new<T: Named>(named: &T, ty: syn::Type) -> Result<Self, syn::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ty,
            for_token: Default::default(),
            ident: named.name()?
        })
    }

    pub fn from_bytes(ir: &TypeIR) -> Result<Self, syn::Error> {
        Self::new(ir, utils::ty::from_bytes())
    }

    pub fn to_bytes(ir: &TypeIR) -> Result<Self, syn::Error> {
        Self::new(ir, utils::ty::to_bytes())
    }

    pub fn cl_typed(ir: &TypeIR) -> Result<Self, syn::Error> {
        Self::new(ir, utils::ty::cl_typed())
    }

    pub fn clone<T: Named>(named: &T) -> Result<Self, syn::Error> {
        Self::new(named, utils::ty::clone())
    }
}

pub trait Named {
    fn name(&self) -> Result<syn::Ident, syn::Error>;
}

impl Named for TypeIR {
    fn name(&self) -> Result<syn::Ident, syn::Error> {
        Ok(self.self_code().ident.clone())
    }
}

impl Named for StructIR {
    fn name(&self) -> Result<syn::Ident, syn::Error> {
        Ok(self.module_ident())
    }
}
