use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::parse::Parse;

pub(crate) fn generate_code(item: proc_macro::TokenStream) -> TokenStream {
    let item_enum: ErrorEnumItem = syn::parse2(item.into()).unwrap();

    let enum_ident = &item_enum.ident;
    quote::quote! { 
        #item_enum 

        impl Into<odra::types::error::ExecutionError> for #enum_ident {
            fn into(self) -> odra::types::error::ExecutionError {
                match self {
                    #enum_ident::Dupa => odra::types::error::ExecutionError::new(3, "Not an owner"),
                    #enum_ident::Kupa => odra::types::error::ExecutionError::new(4, "Owner is not initialized."),
                }
            }
        }
    }
}


struct ErrorEnumItem {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub enum_token: syn::Token![enum],
    pub ident: syn::Ident,
    pub generics: syn::Generics,
    pub brace_token: syn::token::Brace,
    pub variants: syn::punctuated::Punctuated<Variant, syn::Token![,]>,
}

impl Parse for ErrorEnumItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
            let vis = input.parse::<syn::Visibility>()?;
            let enum_token = input.parse::<syn::Token![enum]>()?;
            let ident = input.parse::<syn::Ident>()?;
            let generics = input.parse::<syn::Generics>()?;

            let where_clause = input.parse()?;
            let content;
            let brace_token = syn::braced!(content in input);
            let variants = content.parse_terminated(Variant::parse)?;

            Ok(ErrorEnumItem {
                attrs,
                vis,
                enum_token,
                ident,
                generics: syn::Generics {
                    where_clause,
                    ..generics
                },
                brace_token,
                variants,
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

pub struct Variant {
    /// Name of the variant.
    pub ident: syn::Ident,
    pub fat_arrow_token: syn::Token![=>],
    pub expr: syn::Expr,
}

impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _attrs = input.call(syn::Attribute::parse_outer)?;
        let _visibility: syn::Visibility = input.parse()?;
        let ident: syn::Ident = input.parse()?;
        dbg!(ident.clone());
        // dbg!(input);
        let fat_arrow_token: syn::Token![=>] = input.parse()?;
        dbg!(fat_arrow_token.clone());
        let expr: syn::Expr = input.parse()?;
        dbg!(expr.clone());
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
        // self.fat_arrow_token.to_tokens(tokens);
        // self.expr.to_tokens(tokens);
    }
}