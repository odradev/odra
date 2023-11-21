use quote::{quote, ToTokens};

pub struct UsePreludeItem;

impl ToTokens for UsePreludeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use odra::prelude::*;
        ));
    }
}

pub struct UseSuperItem;

impl ToTokens for UseSuperItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use super::*;
        ));
    }
}
