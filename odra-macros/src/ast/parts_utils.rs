use quote::{quote, ToTokens, format_ident};

use crate::{ir::ModuleIR, utils};

pub struct UsePreludeItem;

impl ToTokens for UsePreludeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use odra2::prelude::*;
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

pub fn test_parts_mod_ident(module: &ModuleIR) -> Result<syn::Ident, syn::Error> {
    module
        .module_ident()
        .map(utils::string::to_lower_case)
        .map(|ident| format_ident!("__{}_test_parts", ident))
}
