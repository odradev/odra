use crate::ir::{errors, ModuleStructIR, TypeIR};

/// The ident of the item a macro is applied to.
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
