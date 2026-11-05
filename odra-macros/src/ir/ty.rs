use proc_macro2::{Ident, Span};
use syn::spanned::Spanned;

use super::errors;
use crate::utils;

/// The struct or enum `#[odra::odra_type]` or `#[odra::odra_error]` is applied to: a struct
/// with named fields, or an enum whose explicit discriminants are `u16` literals.
pub struct TypeIR {
    code: syn::Item,
    ident: Ident,
    kind: TypeKind
}

/// The IR from the macro's token stream, for the test mocks.
#[cfg(test)]
impl TryFrom<&proc_macro2::TokenStream> for TypeIR {
    type Error = syn::Error;

    fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Self::try_new(syn::parse2(stream.clone())?)
    }
}

impl TypeIR {
    pub fn try_new(code: syn::Item) -> syn::Result<Self> {
        let (ident, kind) = match &code {
            syn::Item::Enum(e) => {
                let variants = VariantIR::try_all(&e.variants)?;
                let is_unit = variants.iter().all(|v| v.fields.is_empty());
                let kind = match is_unit {
                    true => TypeKind::UnitEnum { variants },
                    false => TypeKind::Enum { variants }
                };
                (e.ident.clone(), kind)
            }
            syn::Item::Struct(syn::ItemStruct { ident, fields, .. }) => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        f.ident
                            .clone()
                            .map(|i| (i, f.ty.clone()))
                            .ok_or_else(|| errors::type_fields_unnamed(fields))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                (ident.clone(), TypeKind::Struct { fields })
            }
            _ => return Err(errors::type_on_other_item(&code))
        };
        Ok(Self { code, ident, kind })
    }

    pub fn self_code(&self) -> &syn::Item {
        &self.code
    }

    pub fn ident(&self) -> Ident {
        self.ident.clone()
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }

    /// The variants of an `#[odra::odra_error]` enum: bare names, as an error is only its code.
    pub fn error_variants(&self) -> syn::Result<&[VariantIR]> {
        let variants = match &self.kind {
            TypeKind::UnitEnum { variants } | TypeKind::Enum { variants } => variants,
            TypeKind::Struct { .. } => return Err(errors::error_on_struct(&self.code))
        };
        errors::check_all(variants.iter().map(|v| match v.fields.is_empty() {
            true => Ok(()),
            false => Err(errors::error_variant_with_data(&v.ident, v.span))
        }))?;
        Ok(variants)
    }
}

/// What the type is, with the parts the emitters read.
#[derive(Clone)]
pub enum TypeKind {
    /// An enum of bare variants: one byte on the wire, its discriminant.
    UnitEnum {
        variants: Vec<VariantIR>
    },
    /// An enum with at least one variant carrying fields.
    Enum {
        variants: Vec<VariantIR>
    },
    Struct {
        fields: Vec<(Ident, syn::Type)>
    }
}

/// A variant of the enum with the code Rust gives it: its discriminant when written, else the
/// previous variant's code plus one. The code is the error's code and the tag of a serialized
/// unit enum.
#[derive(Clone)]
pub struct VariantIR {
    pub ident: Ident,
    pub fields: syn::Fields,
    pub code: u16,
    pub docs: Vec<String>,
    span: Span
}

impl VariantIR {
    /// Every variant with its code; every discriminant that is not a `u16` literal is reported.
    fn try_all<'a>(variants: impl IntoIterator<Item = &'a syn::Variant>) -> syn::Result<Vec<Self>> {
        let variants = variants.into_iter().collect::<Vec<_>>();
        let discriminants = errors::collect_all(variants.iter().map(|v| discriminant(v)))?;
        let mut next = 0u16;
        Ok(variants
            .into_iter()
            .zip(discriminants)
            .map(|(v, discriminant)| {
                let code = discriminant.unwrap_or(next);
                next = code.wrapping_add(1);
                Self {
                    ident: v.ident.clone(),
                    fields: v.fields.clone(),
                    code,
                    docs: utils::syn::string_docs(&v.attrs),
                    span: v.span()
                }
            })
            .collect())
    }
}

/// The variant's discriminant when written, which has to be a `u16` literal.
fn discriminant(variant: &syn::Variant) -> syn::Result<Option<u16>> {
    let Some((_, discriminant)) = &variant.discriminant else {
        return Ok(None);
    };
    let value = match discriminant {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) => int.base10_parse::<u16>().ok(),
        _ => None
    };
    value
        .map(Some)
        .ok_or_else(|| errors::discriminant_not_u16(discriminant))
}
