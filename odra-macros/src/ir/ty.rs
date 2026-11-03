use super::errors;

/// The struct or enum `#[odra::odra_type]` or `#[odra::odra_error]` is applied to.
pub struct TypeIR {
    code: syn::Item
}

impl TryFrom<&proc_macro2::TokenStream> for TypeIR {
    type Error = syn::Error;

    fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            code: syn::parse2::<syn::Item>(stream.clone())?
        })
    }
}

impl TypeIR {
    pub fn self_code(&self) -> &syn::Item {
        &self.code
    }

    pub fn kind(&self) -> syn::Result<TypeKind> {
        match &self.code {
            syn::Item::Enum(e) => {
                errors::check_all(e.variants.iter().map(validate_discriminant))?;
                let is_unit = e.variants.iter().all(|v| v.fields.is_empty());
                let variants = e.variants.iter().cloned().collect();
                if is_unit {
                    Ok(TypeKind::UnitEnum { variants })
                } else {
                    Ok(TypeKind::Enum { variants })
                }
            }
            syn::Item::Struct(syn::ItemStruct { fields, .. }) => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        f.ident
                            .clone()
                            .map(|i| (i, f.ty.clone()))
                            .ok_or_else(|| errors::type_fields_unnamed(fields))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeKind::Struct { fields })
            }
            _ => Err(errors::type_on_other_item(&self.code))
        }
    }
}

/// A variant's discriminant, when given, is a `u16` literal: it is the code of an error and the
/// tag of a serialized enum.
fn validate_discriminant(variant: &syn::Variant) -> syn::Result<()> {
    let Some((_, discriminant)) = &variant.discriminant else {
        return Ok(());
    };
    let is_u16 = match discriminant {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => int.base10_parse::<u16>().is_ok(),
        _ => false
    };
    match is_u16 {
        true => Ok(()),
        false => Err(errors::discriminant_not_u16(discriminant))
    }
}

/// What the type is, with the parts the emitters read.
pub enum TypeKind {
    UnitEnum {
        variants: Vec<syn::Variant>
    },
    Enum {
        variants: Vec<syn::Variant>
    },
    Struct {
        fields: Vec<(syn::Ident, syn::Type)>
    }
}
