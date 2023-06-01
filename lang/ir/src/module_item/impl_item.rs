use std::convert::TryFrom;

use proc_macro2::Ident;
use syn::{punctuated::Punctuated, token::Comma};

use crate::attrs::partition_attributes;

use super::{constructor::Constructor, method::Method, utils};

/// An item within an implementation block
///
/// At this point there is not difference between a [Method] and a default syn::ImplItem
pub enum ImplItem {
    /// A `#[odra(init)]` marked function.
    Constructor(Constructor),
    /// Unmarked function.
    Method(Method),
    /// Any other implementation block item.
    Other(syn::ImplItem)
}

impl quote::ToTokens for ImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Constructor(constructor) => constructor.to_tokens(tokens),
            Self::Method(method) => method.to_tokens(tokens),
            Self::Other(other) => other.to_tokens(tokens)
        }
    }
}

impl TryFrom<syn::ImplItem> for ImplItem {
    type Error = syn::Error;

    fn try_from(value: syn::ImplItem) -> Result<Self, Self::Error> {
        match value {
            syn::ImplItem::Method(method) => {
                let (odra_attrs, _) = partition_attributes(method.attrs.clone())?;
                if odra_attrs.is_empty() {
                    return Ok(ImplItem::Method(method.try_into()?));
                }
                let is_constructor = odra_attrs.iter().any(|attr| attr.is_constructor());
                match is_constructor {
                    true => Ok(ImplItem::Constructor(Constructor::try_from(method)?)),
                    false => Ok(ImplItem::Method(method.try_into()?))
                }
            }
            other_item => Ok(ImplItem::Other(other_item))
        }
    }
}

pub struct ContractEntrypoint {
    pub ident: Ident,
    pub args: Punctuated<syn::PatType, Comma>,
    pub ret: syn::ReturnType,
    pub full_sig: syn::Signature
}

impl From<syn::ImplItemMethod> for ContractEntrypoint {
    fn from(method: syn::ImplItemMethod) -> Self {
        let ident = method.sig.ident.to_owned();
        let args = utils::extract_typed_inputs(&method.sig);
        let ret = method.clone().sig.output;
        let full_sig = method.sig;
        Self {
            ident,
            args,
            ret,
            full_sig
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use super::ImplItem;

    macro_rules! assert_enum_variant {
        ($v:expr, $p:pat) => {
            assert!(if let $p = $v { true } else { false })
        };
    }

    #[test]
    fn test_parse_fn_without_odra_attr() {
        let item: syn::ImplItem = syn::parse_quote! {
            #[some(a)]
            pub fn set_initial_value(&self, value: u32) {
                self.set_value(value);
            }
        };
        let parsed = ImplItem::try_from(item);
        assert_enum_variant!(parsed.unwrap(), ImplItem::Method(_));
    }

    #[test]
    fn test_parse_fn_without_any_attr() {
        let item: syn::ImplItem = syn::parse_quote! {
            pub fn set_initial_value(&self, value: u32) {
                self.set_value(value);
            }
        };
        let parsed = ImplItem::try_from(item);
        assert_enum_variant!(parsed.unwrap(), ImplItem::Method(_));
    }

    #[test]
    fn test_parse_fn_with_odra_init_attr() {
        let item: syn::ImplItem = syn::parse_quote! {
            #[odra(init)]
            pub fn set_initial_value(&self, value: u32) {
                self.set_value(value);
            }
        };
        let parsed = ImplItem::try_from(item);
        assert_enum_variant!(parsed.unwrap(), ImplItem::Constructor(_));
    }

    #[test]
    fn test_parse_other_impl_item() {
        let item: syn::ImplItem = syn::parse_quote! {
            const A: i32 = 3;
        };
        let parsed = ImplItem::try_from(item);
        assert_enum_variant!(parsed.unwrap(), ImplItem::Other(_));
    }
}
