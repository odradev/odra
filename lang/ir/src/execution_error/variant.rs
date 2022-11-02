use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned};

/// Custom enum variant similar to [syn::Variant].
pub struct Variant {
    pub ident: syn::Ident,
    pub fat_arrow_token: syn::Token![=>],
    pub expr: syn::Expr
}

impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(syn::Attribute::parse_outer)?;
        let _visibility: syn::Visibility = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        let fat_arrow_token: syn::Token![=>] = input.parse()?;
        let expr: syn::Expr = input.parse()?;

        let expr: syn::Expr = match expr {
            syn::Expr::Lit(_) => expr,
            _ => {
                return Err(syn::Error::new(
                    expr.span(),
                    "The expression must be of type `syn::Expr::Lit`"
                ))
            }
        };

        Ok(Variant {
            ident,
            fat_arrow_token,
            expr
        })
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
    }
}
