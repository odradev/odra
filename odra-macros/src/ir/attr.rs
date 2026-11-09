use quote::ToTokens;
use syn::{punctuated::Punctuated, Meta, Path, Token};

use super::errors;

/// The `#[odra(..)]` attributes of a function, parsed: each attribute with the flags it
/// carries, in source order. A flag is given at most once across all of them.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OdraAttrs {
    attrs: Vec<(syn::Attribute, Vec<OdraFlag>)>
}

impl OdraAttrs {
    /// Parses the `#[odra(..)]` attributes among `attrs`, failing at the first unknown,
    /// malformed or duplicated flag.
    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut parsed = Self::default();
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("odra")) {
            let flags = parsed.parse_attribute(attr)?;
            parsed.attrs.push((attr.clone(), flags));
        }
        Ok(parsed)
    }

    /// The flags of one `#[odra(..)]`, checked against the ones already read.
    fn parse_attribute(&self, attr: &syn::Attribute) -> syn::Result<Vec<OdraFlag>> {
        let metas = match &attr.meta {
            Meta::List(list) => {
                list.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?
            }
            _ => return Err(errors::odra_attr_malformed(attr, &OdraFlag::expected()))
        };
        if metas.is_empty() {
            return Err(errors::odra_attr_empty(attr, &OdraFlag::expected()));
        }
        let mut flags = Vec::with_capacity(metas.len());
        for meta in metas.iter() {
            let flag = OdraFlag::try_from(meta)?;
            if self.has(flag) || flags.contains(&flag) {
                return Err(errors::odra_attr_duplicate(meta, flag.name()));
            }
            flags.push(flag);
        }
        Ok(flags)
    }

    fn has(&self, flag: OdraFlag) -> bool {
        self.attrs.iter().any(|(_, flags)| flags.contains(&flag))
    }

    pub fn is_payable(&self) -> bool {
        self.has(OdraFlag::Payable)
    }

    pub fn is_non_reentrant(&self) -> bool {
        self.has(OdraFlag::NonReentrant)
    }

    pub fn is_offchain(&self) -> bool {
        self.has(OdraFlag::Offchain)
    }

    /// The attribute that says `offchain`, for an error about it.
    pub fn offchain_attribute(&self) -> Option<&syn::Attribute> {
        self.attrs
            .iter()
            .find(|(_, flags)| flags.contains(&OdraFlag::Offchain))
            .map(|(attr, _)| attr)
    }

    /// Each attribute with the name of its first flag, for an error about the attribute.
    pub fn each_attribute(&self) -> impl Iterator<Item = (&syn::Attribute, &'static str)> {
        // An attribute has at least one flag: an empty one is rejected when parsed.
        self.attrs
            .iter()
            .filter_map(|(attr, flags)| Some((attr, flags.first()?.name())))
    }
}

/// A flag of `#[odra(..)]`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum OdraFlag {
    Payable,
    NonReentrant,
    Offchain
}

impl OdraFlag {
    const ALL: [OdraFlag; 3] = [
        OdraFlag::Payable,
        OdraFlag::NonReentrant,
        OdraFlag::Offchain
    ];

    fn name(self) -> &'static str {
        match self {
            OdraFlag::Payable => "payable",
            OdraFlag::NonReentrant => "non_reentrant",
            OdraFlag::Offchain => "offchain"
        }
    }

    /// The accepted flags, for error messages: "payable, non_reentrant, offchain".
    fn expected() -> String {
        Self::ALL
            .iter()
            .map(|flag| flag.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl TryFrom<&'_ syn::Meta> for OdraFlag {
    type Error = syn::Error;

    fn try_from(meta: &'_ syn::Meta) -> Result<Self, Self::Error> {
        let known = |path: &Path| {
            Self::ALL
                .into_iter()
                .find(|flag| path.is_ident(flag.name()))
        };
        let path_str = |path: &Path| path.to_token_stream().to_string().replace(' ', "");
        match meta {
            Meta::Path(path) => known(path)
                .ok_or_else(|| errors::odra_attr_unknown(meta, &path_str(path), &Self::expected())),
            Meta::List(syn::MetaList { path, .. })
            | Meta::NameValue(syn::MetaNameValue { path, .. }) => Err(match known(path) {
                Some(flag) => errors::odra_attr_with_value(meta, flag.name()),
                None => errors::odra_attr_unknown(meta, &path_str(path), &Self::expected())
            })
        }
    }
}

/// The attributes other than `#[odra(..)]` (which [OdraAttrs::parse] validates).
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
