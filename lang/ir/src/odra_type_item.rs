use proc_macro2::Ident;
use syn::{DataEnum, DataStruct, DeriveInput, Fields, Variant};

/// Odra Type definition.
pub enum OdraTypeItem {
    Struct(OdraStruct),
    Enum(OdraEnum)
}

pub struct OdraStruct {
    struct_ident: Ident,
    fields: Vec<Ident>
}

impl OdraStruct {
    pub fn struct_ident(&self) -> &Ident {
        &self.struct_ident
    }

    pub fn fields(&self) -> &[Ident] {
        self.fields.as_ref()
    }
}

pub struct OdraEnum {
    enum_ident: Ident,
    variants: Vec<Variant>
}

impl OdraEnum {
    pub fn enum_ident(&self) -> &Ident {
        &self.enum_ident
    }

    pub fn variants(&self) -> &[Variant] {
        &self.variants
    }
}

impl OdraTypeItem {
    pub fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let ident = input.ident.clone();
        match &input.data {
            syn::Data::Struct(DataStruct {
                fields: Fields::Named(named_fields),
                ..
            }) => Ok(OdraTypeItem::Struct(OdraStruct {
                struct_ident: ident,
                fields: named_fields
                    .named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect()
            })),
            syn::Data::Enum(DataEnum { variants, .. }) => {
                let is_valid = variants
                    .iter()
                    .all(|v| matches!(v.fields, syn::Fields::Unit));
                if is_valid {
                    Ok(OdraTypeItem::Enum(OdraEnum {
                        enum_ident: ident,
                        variants: variants.iter().map(Clone::clone).collect()
                    }))
                } else {
                    Err(syn::Error::new_spanned(
                        input,
                        "Expected a unit enum variant."
                    ))
                }
            }
            _ => Err(syn::Error::new_spanned(
                input,
                "Expected a struct or enum with named fields."
            ))
        }
    }

    pub fn ident(&self) -> &Ident {
        match self {
            OdraTypeItem::Struct(s) => s.struct_ident(),
            OdraTypeItem::Enum(e) => e.enum_ident()
        }
    }
}
