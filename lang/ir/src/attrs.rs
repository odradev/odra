use std::{collections::HashSet, convert::TryFrom};

use itertools::{Either, Itertools};
use proc_macro2::{Ident, Span};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Attribute {
    Odra(OdraAttribute),
    Other(syn::Attribute),
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
    kinds: Vec<AttrKind>,
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
                _ => Err(syn::Error::new_spanned(attrs, "unknown Odra attr")),
            })
            .unwrap()
            .unwrap();

        ensure_no_duplicates(kinds.clone())?;

        Ok(OdraAttribute { kinds })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum AttrKind {
    Constructor,
    Entrypoint,
    Payable,
}

impl TryFrom<syn::NestedMeta> for AttrKind {
    type Error = syn::Error;

    fn try_from(nested_meta: syn::NestedMeta) -> Result<Self, Self::Error> {
        match nested_meta {
            syn::NestedMeta::Meta(meta) => match &meta {
                syn::Meta::Path(path) => path
                    .get_ident()
                    .map(Ident::to_string)
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&meta, "unknown Odra attribute argument (path)")
                    })
                    .and_then(|ident| match ident.as_str() {
                        "init" => Ok(AttrKind::Constructor),
                        "payable" => Ok(AttrKind::Payable),
                        "entrypoint" => Ok(AttrKind::Entrypoint),
                        _ => Err(syn::Error::new_spanned(
                            meta,
                            "unknown Odra attribute argument (path)",
                        )),
                    }),
                syn::Meta::List(_) => Err(syn::Error::new_spanned(
                    meta,
                    "unknown Odra attribute argument (list)",
                )),
                syn::Meta::NameValue(_) => Err(syn::Error::new_spanned(
                    meta,
                    "unknown Odra attribute argument (name = value)",
                )),
            },
            syn::NestedMeta::Lit(_) => Err(syn::Error::new_spanned(
                nested_meta,
                "unknown Odra attribute argument (literal)",
            )),
        }
    }
}

fn ensure_no_duplicates<I>(attrs: I) -> Result<(), syn::Error>
where
    I: IntoIterator<Item = AttrKind>,
{
    let mut set: HashSet<AttrKind> = HashSet::new();

    let contains_duplicate = attrs.into_iter().any(|attr| !set.insert(attr));
    match contains_duplicate {
        true => Err(syn::Error::new(
            Span::call_site(),
            "attr duplicate encountered".to_string(),
        )),
        false => Ok(()),
    }
}

pub fn partition_attributes<I>(
    attrs: I,
) -> Result<(Vec<OdraAttribute>, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>,
{
    let (odra_attrs, other_attrs): (Vec<OdraAttribute>, Vec<syn::Attribute>) = attrs
        .into_iter()
        .map(<Attribute as TryFrom<_>>::try_from)
        .collect::<Result<Vec<Attribute>, syn::Error>>()?
        .into_iter()
        .partition_map(|attr| match attr {
            Attribute::Odra(odra_attr) => Either::Left(odra_attr),
            Attribute::Other(other_attr) => Either::Right(other_attr),
        });

    ensure_no_duplicates(odra_attrs.clone().into_iter().flat_map(|attr| attr.kinds))?;
    Ok((odra_attrs, other_attrs))
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    #[test]
    fn constructor_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Constructor],
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(init)]
            },
            Ok(expected_value),
        );
    }

    #[test]
    fn payable_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Payable],
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(payable)]
            },
            Ok(expected_value),
        );
    }

    #[test]
    fn multiple_attr_works() {
        let expected_value = Attribute::Odra(OdraAttribute {
            kinds: vec![AttrKind::Constructor, AttrKind::Payable],
        });
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(init, payable)]
            },
            Ok(expected_value.clone()),
        );

        let expected_value = vec![
            Attribute::Odra(OdraAttribute {
                kinds: vec![AttrKind::Constructor],
            }),
            Attribute::Odra(OdraAttribute {
                kinds: vec![AttrKind::Payable],
            }),
        ];
        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(init)] },
                syn::parse_quote! { #[odra(payable)] },
            ],
            Ok((expected_value, vec![])),
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
            Err("attr duplicate encountered"),
        );

        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(init)] },
                syn::parse_quote! { #[odra(init)] },
            ],
            Err("attr duplicate encountered"),
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
        expected: Result<(Vec<Attribute>, Vec<syn::Attribute>), &'static str>,
    ) {
        let result = partition_attributes(inputs)
            .map(|(odra_attrs, other_attrs)| {
                (
                    odra_attrs
                        .into_iter()
                        .map(Attribute::from)
                        .collect::<Vec<_>>(),
                    other_attrs,
                )
            })
            .map_err(|err| err.to_string());
        assert_eq!(result, expected.map_err(ToString::to_string));
    }
}
