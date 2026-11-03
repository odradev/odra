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

// Wrong attributes (unknown, duplicate, malformed) are covered by the compile-fail tests of
// the `odra` crate (`tests/macros/fail`), which check the message and its span.

fn assert_attribute_try_from(input: syn::Attribute, expected: Result<Attribute, &'static str>) {
    assert_eq!(
        <Attribute as TryFrom<_>>::try_from(input).map_err(|err| err.to_string()),
        expected.map_err(ToString::to_string),
    );
}
