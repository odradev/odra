use std::{collections::HashSet, convert::TryFrom};

use quote::ToTokens;
use syn::{punctuated::Punctuated, Meta, Path, Token};

use super::errors;

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
        _ => return Err(errors::odra_attr_malformed(attr, &AttrType::expected()))
    };
    if metas.is_empty() {
        return Err(errors::odra_attr_empty(attr, &AttrType::expected()));
    }
    let mut types = Vec::with_capacity(metas.len());
    for meta in metas.iter() {
        let ty = AttrType::try_from(meta)?;
        if !seen.insert(ty.clone()) {
            return Err(errors::odra_attr_duplicate(meta, ty.name()));
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
            Meta::Path(path) => known(path)
                .ok_or_else(|| errors::odra_attr_unknown(meta, &path_str(path), &Self::expected())),
            Meta::List(syn::MetaList { path, .. })
            | Meta::NameValue(syn::MetaNameValue { path, .. }) => Err(match known(path) {
                Some(ty) => errors::odra_attr_with_value(meta, ty.name()),
                None => errors::odra_attr_unknown(meta, &path_str(path), &Self::expected())
            })
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
mod tests;
