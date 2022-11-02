use syn::{spanned::Spanned, DeriveInput};

/// Odra module instance definition.
///
/// Only a struct can be an instance of Odra module.
pub struct InstanceItem {
    ident: syn::Ident,
    data_struct: syn::DataStruct
}

impl InstanceItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        match input.data {
            syn::Data::Struct(data_struct) => Ok(Self {
                ident: input.ident,
                data_struct
            }),
            _ => Err(syn::Error::new(
                input.span(),
                "Only struct can derive from Instance"
            ))
        }
    }

    pub fn data_struct(&self) -> &syn::DataStruct {
        &self.data_struct
    }

    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}
