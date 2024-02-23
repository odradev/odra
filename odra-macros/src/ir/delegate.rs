use crate::utils;
use quote::ToTokens;

mod kw {
    syn::custom_keyword!(to);
}

#[derive(Debug, Clone)]
pub struct Delegate {
    pub functions: Vec<syn::ImplItemFn>
}

impl syn::parse::Parse for Delegate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut functions = Vec::new();
        while !input.is_empty() {
            input.parse::<kw::to>()?;
            let delegate_to = input.parse::<syn::ExprField>()?;
            let content;
            let _brace_token = syn::braced!(content in input);
            while !content.is_empty() {
                let fn_item = content.parse::<syn::TraitItemFn>()?;
                let fn_ident = fn_item.sig.ident.clone();
                let args = utils::syn::function_typed_args(&fn_item.sig);
                let args = args
                    .iter()
                    .filter_map(|ty| match &*ty.pat {
                        syn::Pat::Ident(pat) => Some(pat.ident.clone()),
                        _ => None
                    })
                    .collect::<syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>>();

                let mut attrs = fn_item.attrs.clone();

                let comment = format!(
                    " Delegated. See `{}.{}.{}()` for details.",
                    delegate_to.base.to_token_stream(),
                    delegate_to.member.to_token_stream(),
                    fn_ident
                );

                let attr = syn::parse_quote!(#[doc = #comment]);
                attrs.push(attr);

                functions.push(syn::ImplItemFn {
                    attrs,
                    vis: utils::syn::visibility_pub(),
                    defaultness: None,
                    sig: fn_item.sig,
                    block: syn::parse_quote!({ #delegate_to.#fn_ident(#args) })
                });
            }
        }
        Ok(Self { functions })
    }
}
