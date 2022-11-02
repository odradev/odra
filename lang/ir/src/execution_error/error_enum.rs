use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned};

use super::variant::Variant;

/// User defined contract error with all required information.
///
/// The structure is similar to [syn::ItemEnum], but contains custom variants.
pub struct ErrorEnumItem {
    pub vis: syn::Visibility,
    pub enum_token: syn::Token![enum],
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub brace_token: syn::token::Brace,
    pub variants: syn::punctuated::Punctuated<Variant, syn::Token![,]>
}

impl ErrorEnumItem {
    pub fn parse(input: TokenStream) -> Result<Self, syn::Error> {
        syn::parse2::<ErrorEnumItem>(input)
    }
}

impl Parse for ErrorEnumItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<syn::Visibility>()?;
        let enum_token = input.parse::<syn::Token![enum]>()?;
        let ident = input.parse::<syn::Ident>()?;
        let generics = input.parse::<syn::Generics>()?;

        let where_clause = input.parse()?;
        let content;
        let brace_token = syn::braced!(content in input);
        let variants = content.parse_terminated(Variant::parse)?;

        let unique_variant_count = variants.iter().map(|v| &v.expr).unique().count();
        let variant_count = variants.iter().count();

        if unique_variant_count != variant_count {
            return Err(syn::Error::new(
                variants.span(),
                "Each error must have a unique code."
            ));
        }

        Ok(ErrorEnumItem {
            vis,
            enum_token,
            ident,
            generics: syn::Generics {
                where_clause,
                ..generics
            },
            brace_token,
            variants
        })
    }
}

impl ToTokens for ErrorEnumItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.vis.to_tokens(tokens);
        self.enum_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        self.generics.where_clause.to_tokens(tokens);
        self.brace_token.surround(tokens, |tokens| {
            self.variants.to_tokens(tokens);
        });
    }
}
