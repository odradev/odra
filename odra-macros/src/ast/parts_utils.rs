use quote::{quote, ToTokens};

#[derive(Default)]
pub struct UsePreludeItem;

impl ToTokens for UsePreludeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use odra::prelude::*;
        ));
    }
}

#[derive(Default)]
pub struct UseSuperItem;

impl ToTokens for UseSuperItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use super::*;
        ));
    }
}
