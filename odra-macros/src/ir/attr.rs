use std::{collections::HashSet, convert::TryFrom};

use quote::ToTokens;
use syn::{punctuated::Punctuated, Meta, Path, Token};

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

    pub fn is_offchain(&self) -> bool {
        self.types
            .iter()
            .any(|attr_kind| matches!(attr_kind, &AttrType::Offchain))
    }

    /// Names of the arguments this attribute carries, as written by the user,
    /// eg. `["payable"]` for `#[odra(payable)]`.
    pub fn arg_names(&self) -> Vec<&'static str> {
        self.types.iter().map(AttrType::name).collect()
    }
}

impl TryFrom<syn::Attribute> for OdraAttribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        let mut seen = HashSet::new();
        parse_odra_attribute(&attr, &mut seen)
    }
}

/// Parses the arguments of a single `#[odra(..)]`, rejecting unknown, malformed and duplicated
/// ones (`seen` carries the arguments of the previous `#[odra(..)]` attributes of the same item).
fn parse_odra_attribute(
    attr: &syn::Attribute,
    seen: &mut HashSet<AttrType>
) -> syn::Result<OdraAttribute> {
    let metas = match &attr.meta {
        Meta::List(list) => {
            list.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?
        }
        _ => {
            return Err(syn::Error::new_spanned(
                attr,
                format!(
                    "malformed odra attribute, expected `#[odra(..)]` with one or more of: {}",
                    AttrType::expected()
                )
            ))
        }
    };
    if metas.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            format!(
                "empty odra attribute, expected one or more of: {}",
                AttrType::expected()
            )
        ));
    }
    let mut types = Vec::with_capacity(metas.len());
    for meta in metas.iter() {
        let ty = AttrType::try_from(meta)?;
        if !seen.insert(ty.clone()) {
            return Err(syn::Error::new_spanned(
                meta,
                format!("duplicate odra attribute `{}`", ty.name())
            ));
        }
        types.push(ty);
    }
    Ok(OdraAttribute { types })
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum AttrType {
    Payable,
    NonReentrant,
    Offchain
}

impl AttrType {
    const ALL: [AttrType; 3] = [
        AttrType::Payable,
        AttrType::NonReentrant,
        AttrType::Offchain
    ];

    fn name(&self) -> &'static str {
        match self {
            AttrType::Payable => "payable",
            AttrType::NonReentrant => "non_reentrant",
            AttrType::Offchain => "offchain"
        }
    }

    /// The accepted arguments, for error messages: "payable, non_reentrant, offchain".
    fn expected() -> String {
        Self::ALL
            .iter()
            .map(Self::name)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl TryFrom<&'_ syn::Meta> for AttrType {
    type Error = syn::Error;

    fn try_from(meta: &'_ syn::Meta) -> Result<Self, Self::Error> {
        let known = |path: &Path| Self::ALL.into_iter().find(|ty| path.is_ident(ty.name()));
        let path_str = |path: &Path| path.to_token_stream().to_string().replace(' ', "");
        match meta {
            Meta::Path(path) => known(path).ok_or_else(|| {
                syn::Error::new_spanned(
                    meta,
                    format!(
                        "unknown odra attribute `{}`, expected one of: {}",
                        path_str(path),
                        Self::expected()
                    )
                )
            }),
            Meta::List(syn::MetaList { path, .. })
            | Meta::NameValue(syn::MetaNameValue { path, .. }) => {
                let msg = match known(path) {
                    Some(ty) => format!("odra attribute `{}` takes no value", ty.name()),
                    None => format!(
                        "unknown odra attribute `{}`, expected one of: {}",
                        path_str(path),
                        Self::expected()
                    )
                };
                Err(syn::Error::new_spanned(meta, msg))
            }
        }
    }
}

/// Splits the attributes of an item into the parsed `#[odra(..)]` ones and the others.
///
/// Fails at the span of the first unknown, malformed or duplicated `#[odra(..)]` argument; the
/// duplicate check spans all the `#[odra(..)]` attributes of the item.
pub fn partition_attributes<I>(attrs: I) -> syn::Result<(Vec<OdraAttribute>, Vec<syn::Attribute>)>
where
    I: IntoIterator<Item = syn::Attribute>
{
    let mut seen = HashSet::new();
    let mut odra_attrs = Vec::new();
    let mut other_attrs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("odra") {
            odra_attrs.push(parse_odra_attribute(&attr, &mut seen)?);
        } else {
            other_attrs.push(attr);
        }
    }
    Ok((odra_attrs, other_attrs))
}

/// The attributes other than `#[odra(..)]` (which are validated by [partition_attributes]).
pub fn other_attributes<I>(attrs: I) -> Vec<syn::Attribute>
where
    I: IntoIterator<Item = syn::Attribute>
{
    attrs
        .into_iter()
        .filter(|attr| !attr.path().is_ident("odra"))
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fmt::{self, Formatter};

    #[derive(Clone)]
    enum Attribute {
        Odra(OdraAttribute),
        Other(Box<syn::Attribute>)
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
            Ok(Attribute::Other(Box::new(attr)))
        }
    }

    impl From<OdraAttribute> for Attribute {
        fn from(odra_attribute: OdraAttribute) -> Self {
            Attribute::Odra(odra_attribute)
        }
    }

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
    fn offchain_attr_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(offchain)]
            },
            Ok(Attribute::Odra(OdraAttribute {
                types: vec![AttrType::Offchain]
            }))
        );
    }

    #[test]
    fn non_odra_attr_works() {
        let expected_value: syn::Attribute = syn::parse_quote! {
            #[yoyo(abc)]
        };
        assert_attribute_try_from(
            expected_value.clone(),
            Ok(Attribute::Other(Box::new(expected_value)))
        );
    }

    #[test]
    fn duplicated_attrs_fail() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[odra(payable, payable)]
            },
            Err("duplicate odra attribute `payable`")
        );

        assert_attributes_try_from(
            vec![
                syn::parse_quote! { #[odra(payable)] },
                syn::parse_quote! { #[odra(payable)] },
            ],
            Err("duplicate odra attribute `payable`")
        )
    }

    #[test]
    fn unknown_attrs_fail() {
        assert_attribute_try_from(
            syn::parse_quote! { #[odra(paybale)] },
            Err("unknown odra attribute `paybale`, expected one of: payable, non_reentrant, offchain")
        );
        assert_attribute_try_from(
            syn::parse_quote! { #[odra(payable, a::b)] },
            Err("unknown odra attribute `a::b`, expected one of: payable, non_reentrant, offchain")
        );
        assert_attribute_try_from(
            syn::parse_quote! { #[odra(using = "x")] },
            Err(
                "unknown odra attribute `using`, expected one of: payable, non_reentrant, offchain"
            )
        );
    }

    #[test]
    fn malformed_attrs_fail() {
        assert_attribute_try_from(
            syn::parse_quote! { #[odra(payable = true)] },
            Err("odra attribute `payable` takes no value")
        );
        assert_attribute_try_from(
            syn::parse_quote! { #[odra(offchain(x))] },
            Err("odra attribute `offchain` takes no value")
        );
        assert_attribute_try_from(
            syn::parse_quote! { #[odra] },
            Err("malformed odra attribute, expected `#[odra(..)]` with one or more of: payable, non_reentrant, offchain")
        );
        assert_attribute_try_from(
            syn::parse_quote! { #[odra()] },
            Err("empty odra attribute, expected one or more of: payable, non_reentrant, offchain")
        );
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
                        .map(|a| Attribute::Other(Box::new(a)))
                        .collect::<Vec<_>>()
                )
            })
            .map_err(|err| err.to_string());
        assert_eq!(result, expected.map_err(ToString::to_string));
    }
}
