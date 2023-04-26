use proc_macro2::Ident;
use syn::{DeriveInput, Field, FieldsNamed};

use crate::utils;

/// Odra Type definition.
pub struct OdraTypeItem {
    struct_ident: Ident,
    named_fields: FieldsNamed
}

impl OdraTypeItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ident = input.ident.clone();
        let named_fields = utils::extract_fields(input)?;

        Ok(Self {
            struct_ident,
            named_fields
        })
    }

    pub fn fields_iter(&self) -> impl Iterator<Item = &Field> {
        self.named_fields.named.iter()
    }

    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }
}
