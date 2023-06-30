use std::{collections::HashSet, convert::TryFrom};

use itertools::{Either, Itertools};
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{Lit, Path};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Attribute {
    Odra(OdraAttribute),
    Other(syn::Attribute)
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if attr.path.is_ident("odra") {
            return <OdraAttribute as TryFrom<_>>::try_from(attr).map(Into::into);
        }
        Ok(Attribute::Other(attr))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OdraAttribute {
    kinds: Vec<AttrKind>
}

impl OdraAttribute {
    pub fn is_constructor(&self) -> bool {
        self.kinds
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrKind::Constructor))
    }

    pub fn is_payable(&self) -> bool {
        self.kinds
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrKind::Payable))
    }

    pub fn is_non_reentrant(&self) -> bool {
        self.kinds
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrKind::NonReentrant))
    }

    pub fn using(&self) -> Vec<String> {
        self.kinds
            .iter()
            .filter_map(|attr| match attr {
                AttrKind::Using(fields) => Some(fields.clone()),
                _ => None
            })
            .flatten()
            .dedup()
            .collect::<Vec<_>>()
    }
}

impl From<OdraAttribute> for Attribute {
    fn from(odra_attribute: OdraAttribute) -> Self {
        Attribute::Odra(odra_attribute)
    }
}

impl TryFrom<syn::Attribute> for OdraAttribute {
    type Error = syn::Error;

    fn try_from(attrs: syn::Attribute) -> Result<Self, Self::Error> {
        let kinds = attrs
            .parse_meta()
            .map(|meta| match meta {
                syn::Meta::List(meta_list) => {
                    let attr_kinds = meta_list
                        .nested
                        .into_iter()
                        .map(<AttrKind as TryFrom<_>>::try_from)
                        .collect::<Result<Vec<_>, syn::Error>>()?;

                    Ok(attr_kinds)
                }
                _ => Err(syn::Error::new_spanned(attrs, "unknown Odra attr"))
            })
            .unwrap()
            .unwrap();

        validate(&kinds)?;

        Ok(OdraAttribute { kinds })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum AttrKind {
    Constructor,
    Entrypoint,
    Payable,
    NonReentrant,
    Using(Vec<String>)
}

impl TryFrom<syn::NestedMeta> for AttrKind {
    type Error = syn::Error;

    fn try_from(nested_meta: syn::NestedMeta) -> Result<Self, Self::Error> {
        match nested_meta {
            syn::NestedMeta::Meta(meta) => match &meta {
                syn::Meta::Path(path) => {
                    path.try_to_string(&meta)
                        .and_then(|ident| match ident.as_str() {
                            "init" => Ok(AttrKind::Constructor),
                            "payable" => Ok(AttrKind::Payable),
                            "entrypoint" => Ok(AttrKind::Entrypoint),
                            "non_reentrant" => Ok(AttrKind::NonReentrant),
                            _ => Err(AttrKindError::Path(&meta).into())
                        })
                }
                syn::Meta::List(_) => Err(AttrKindError::List(&meta).into()),
                syn::Meta::NameValue(name_value) => {
                    let delegated_fields = match &name_value.lit {
                        Lit::Str(str) => str
                            .value()
                            .split(',')
                            .map(|str| str.trim().to_string())
                            .collect::<Vec<_>>(),
                        _ => return Err(AttrKindError::NameValue(&meta).into())
                    };
                    name_value
                        .path
                        .try_to_string(&meta)
                        .and_then(|ident| match ident.as_str() {
                            "using" => Ok(AttrKind::Using(delegated_fields)),
                            _ => Err(AttrKindError::NameValue(&meta).into())
                        })
                }
            },
            syn::NestedMeta::Lit(_) => Err(AttrKindError::Lit(&nested_meta).into())
        }
    }
}

trait TryToString {
    fn try_to_string<T: ToTokens>(&self, span: &T) -> Result<String, syn::Error>;
}

impl TryToString for Path {
    fn try_to_string<T: ToTokens>(&self, span: &T) -> Result<String, syn::Error> {
        self.get_ident()
            .map(Ident::to_string)
            .ok_or_else(|| syn::Error::new_spanned(span, "unknown Odra attribute argument (path)"))
    }
}

enum AttrKindError<'a, T> {
    Lit(&'a T),
    List(&'a T),
    Path(&'a T),
    NameValue(&'a T)
}

impl<T: ToTokens> AttrKindError<'_, T> {
    fn span(&self) -> &T {
        match self {
            AttrKindError::Lit(span) => span,
            AttrKindError::List(span) => span,
            AttrKindError::Path(span) => span,
            AttrKindError::NameValue(span) => span
        }
    }
}

impl<T: ToTokens> From<AttrKindError<'_, T>> for syn::Error {
    fn from(value: AttrKindError<'_, T>) -> Self {
        let ty = match value {
            AttrKindError::Lit(_) => "literal",
            AttrKindError::List(_) => "list",
            AttrKindError::Path(_) => "path",
            AttrKindError::NameValue(_) => "name = value"
        };
        syn::Error::new_spanned(
            value.span(),
            format!("unknown Odra attribute argument ({})", ty)
        )
    }
}

fn ensure_no_duplicates(attrs: &[AttrKind]) -> Result<(), syn::Error> {
    let mut set: HashSet<&AttrKind> = HashSet::new();

    let contains_duplicate = attrs.iter().any(|attr| !set.insert(attr));
    match contains_duplicate {
        true => Err(syn::Error::new(
            Span::call_site(),
            "attr duplicate encountered".to_string()
        )),
        false => Ok(())
    }
}

fn validate(attrs: &[AttrKind]) -> Result<(), syn::Error> {
    let mut has_constructor = false;
    let mut has_payable = false;
    attrs.iter().for_each(|attr| match attr {
        AttrKind::Constructor => has_constructor = true,
        AttrKind::Payable => has_payable = true,
        _ => {}
    });
    if has_constructor && has_payable {
        return Err(syn::Error::new(
            Span::call_site(),
            "constructor cannot be payable".to_string()
        ));
    }

    ensure_no_duplicates(attrs)
}

pub fn partition_attributes<I>(
    attrs: I
) -> Result<(Vec<OdraAttribute>, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>
{
    let (odra_attrs, other_attrs): (Vec<OdraAttribute>, Vec<syn::Attribute>) = attrs
        .into_iter()
        .map(<Attribute as TryFrom<_>>::try_from)
        .collect::<Result<Vec<Attribute>, syn::Error>>()?
        .into_iter()
        .partition_map(|attr| match attr {
            Attribute::Odra(odra_attr) => Either::Left(odra_attr),
            Attribute::Other(other_attr) => Either::Right(other_attr)
        });

    let attrs = odra_attrs
        .clone()
        .into_iter()
        .flat_map(|attr| attr.kinds)
        .collect::<Vec<_>>();
    validate(&attrs)?;
    Ok((odra_attrs, other_attrs))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn constructor_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Constructor]
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(init)]
            },
            Ok(expected_value)
        );
    }

    #[test]
    fn using_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Using(vec![
                String::from("self.value"),
                String::from("self.module"),
            ])]
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(using = "self.value, self.module")]
            },
            Ok(expected_value)
        );
    }

    #[test]
    fn payable_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Payable]
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(payable)]
            },
            Ok(expected_value)
        );
    }

    #[test]
    fn constructor_cannot_be_payable() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(init, payable)]
            },
            Err("constructor cannot be payable")
        );

        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(init)] },
                syn::parse_quote! { #[odra(payable)] },
            ],
            Err("constructor cannot be payable")
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
                #[odra(init, init)]
            },
            Err("attr duplicate encountered")
        );

        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(init)] },
                syn::parse_quote! { #[odra(init)] },
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
        expected: Result<(Vec<Attribute>, Vec<syn::Attribute>), &'static str>
    ) {
        let result = partition_attributes(inputs)
            .map(|(odra_attrs, other_attrs)| {
                (
                    odra_attrs
                        .into_iter()
                        .map(Attribute::from)
                        .collect::<Vec<_>>(),
                    other_attrs
                )
            })
            .map_err(|err| err.to_string());
        assert_eq!(result, expected.map_err(ToString::to_string));
    }
}
