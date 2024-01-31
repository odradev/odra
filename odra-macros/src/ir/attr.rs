use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::{self, Formatter}
};

use itertools::{Either, Itertools};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{punctuated::Punctuated, Meta, Path, Token};

#[derive(Clone)]
pub enum Attribute {
    Odra(OdraAttribute),
    Other(syn::Attribute)
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Attribute::Odra(a), Attribute::Odra(b)) => a == b,
            (Attribute::Other(a), Attribute::Other(b)) => {
                a.to_token_stream().to_string() == b.to_token_stream().to_string()
            }
            _ => false
        }
    }
}

impl std::fmt::Debug for Attribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Odra(odra_attr) => odra_attr.fmt(f),
            Attribute::Other(other_attr) => f
                .debug_struct("syn::Attribute")
                .field("value", &other_attr.to_token_stream().to_string())
                .finish()
        }
    }
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if attr.path().is_ident("odra") {
            return <OdraAttribute as TryFrom<_>>::try_from(attr).map(Into::into);
        }
        Ok(Attribute::Other(attr))
    }
}

impl From<OdraAttribute> for Attribute {
    fn from(odra_attribute: OdraAttribute) -> Self {
        Attribute::Odra(odra_attribute)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OdraAttribute {
    types: Vec<AttrType>
}

impl OdraAttribute {
    pub fn is_payable(&self) -> bool {
        self.types
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrType::Payable))
    }

    pub fn is_non_reentrant(&self) -> bool {
        self.types
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrType::NonReentrant))
    }
}

impl TryFrom<syn::Attribute> for OdraAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        let attrs = attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?;
        let types = attrs
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        ensure_no_duplicates(&types)?;

        Ok(OdraAttribute { types })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum AttrType {
    Payable,
    NonReentrant
}

impl TryFrom<&'_ syn::Meta> for AttrType {
    type Error = syn::Error;

    fn try_from(meta: &'_ syn::Meta) -> Result<Self, Self::Error> {
        match meta {
            Meta::Path(path) => match path.try_to_string(path)?.as_str() {
                "payable" => Ok(AttrType::Payable),
                "non_reentrant" => Ok(AttrType::NonReentrant),
                _ => Err(AttrTypeError::Path(meta).into())
            },
            Meta::List(_) => Err(AttrTypeError::List(meta).into()),
            Meta::NameValue(_) => Err(AttrTypeError::NameValue(meta).into())
        }
    }
}

trait TryToString {
    fn try_to_string<T: ToTokens>(&self, span: &T) -> syn::Result<String>;
}

impl TryToString for Path {
    fn try_to_string<T: ToTokens>(&self, span: &T) -> syn::Result<String> {
        self.get_ident()
            .map(Ident::to_string)
            .ok_or_else(|| syn::Error::new_spanned(span, "unknown Odra attribute argument (path)"))
    }
}

enum AttrTypeError<'a, T> {
    List(&'a T),
    Path(&'a T),
    NameValue(&'a T)
}

impl<T: ToTokens> AttrTypeError<'_, T> {
    fn span(&self) -> &T {
        match self {
            AttrTypeError::List(span) => span,
            AttrTypeError::Path(span) => span,
            AttrTypeError::NameValue(span) => span
        }
    }
}

impl<T: ToTokens> From<AttrTypeError<'_, T>> for syn::Error {
    fn from(value: AttrTypeError<'_, T>) -> Self {
        let ty = match value {
            AttrTypeError::List(_) => "list",
            AttrTypeError::Path(_) => "path",
            AttrTypeError::NameValue(_) => "name = value"
        };
        syn::Error::new_spanned(
            value.span(),
            format!("unknown Odra attribute argument ({})", ty)
        )
    }
}

fn ensure_no_duplicates(types: &[AttrType]) -> syn::Result<()> {
    let mut set: HashSet<&AttrType> = HashSet::new();

    let contains_duplicate = types.iter().any(|attr| !set.insert(attr));
    match contains_duplicate {
        true => Err(syn::Error::new(
            Span::call_site(),
            "attr duplicate encountered".to_string()
        )),
        false => Ok(())
    }
}

pub fn partition_attributes<I>(attrs: I) -> syn::Result<(Vec<OdraAttribute>, Vec<syn::Attribute>)>
where
    I: IntoIterator<Item = syn::Attribute>
{
    let (odra_attrs, other_attrs): (Vec<OdraAttribute>, Vec<syn::Attribute>) = attrs
        .into_iter()
        .map(<Attribute as TryFrom<_>>::try_from)
        .collect::<syn::Result<Vec<Attribute>>>()?
        .into_iter()
        .partition_map(|attr| match attr {
            Attribute::Odra(odra_attr) => Either::Left(odra_attr),
            Attribute::Other(other_attr) => Either::Right(other_attr)
        });

    let attrs = odra_attrs
        .clone()
        .into_iter()
        .flat_map(|attr| attr.types)
        .collect::<Vec<_>>();
    ensure_no_duplicates(&attrs)?;
    Ok((odra_attrs, other_attrs))
}

pub fn other_attributes<I>(attrs: I) -> Vec<syn::Attribute>
where
    I: IntoIterator<Item = syn::Attribute>
{
    let (_, other_attrs) = partition_attributes(attrs).unwrap_or_default();
    other_attrs
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn payable_attr_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(payable)]
            },
            Ok(Attribute::Odra(OdraAttribute {
                types: vec![AttrType::Payable]
            }))
        );
    }

    #[test]
    fn non_reentrant_attr_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(non_reentrant)]
            },
            Ok(Attribute::Odra(OdraAttribute {
                types: vec![AttrType::NonReentrant]
            }))
        );
    }

    #[test]
    fn non_odra_attr_works() {
        let expected_value: syn::Attribute = syn::parse_quote! {
            #[yoyo(abc)]
        };
        assert_attribute_try_from(expected_value.clone(), Ok(Attribute::Other(expected_value)));
    }

    #[test]
    fn duplicated_attrs_fail() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(payable, payable)]
            },
            Err("attr duplicate encountered")
        );

        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(payable)] },
                syn::parse_quote! { #[odra(payable)] },
            ],
            Err("attr duplicate encountered")
        )
    }

    fn assert_attribute_try_from(input: syn::Attribute, expected: Result<Attribute, &'static str>) {
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(input).map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    fn assert_attributes_try_from(
        inputs: Vec<syn::Attribute>,
        expected: Result<(Vec<Attribute>, Vec<Attribute>), &'static str>
    ) {
        let result: Result<(Vec<Attribute>, Vec<Attribute>), String> = partition_attributes(inputs)
            .map(|(odra_attrs, other_attrs)| {
                (
                    odra_attrs
                        .into_iter()
                        .map(Attribute::from)
                        .collect::<Vec<_>>(),
                    other_attrs
                        .into_iter()
                        .map(Attribute::Other)
                        .collect::<Vec<_>>()
                )
            })
            .map_err(|err| err.to_string());
        assert_eq!(result, expected.map_err(ToString::to_string));
    }
}
