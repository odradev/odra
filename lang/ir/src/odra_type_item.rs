use proc_macro2::Ident;
use syn::{DataEnum, DataStruct, DeriveInput};

/// Odra Type definition.
pub struct OdraTypeItem {
    struct_ident: Ident,
    data_struct: Option<DataStruct>,
    data_enum: Option<DataEnum>
}

impl OdraTypeItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let struct_ident = input.ident.clone();
        let data_struct = match &input.data {
            syn::Data::Struct(data_struct) => Some(data_struct.clone()),
            _ => None
        };
        let data_enum = match &input.data {
            syn::Data::Enum(data_enum) => Some(data_enum.clone()),
            _ => None
        };

        if data_enum.is_none() && data_struct.is_none() {
            return Err(syn::Error::new_spanned(
                input,
                "Expected a struct or enum with named fields."
            ));
        }

        Ok(Self {
            struct_ident,
            data_struct,
            data_enum
        })
    }

    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }

    pub fn data_struct(&self) -> Option<&DataStruct> {
        self.data_struct.as_ref()
    }

    pub fn data_enum(&self) -> Option<&DataEnum> {
        self.data_enum.as_ref()
    }
}
