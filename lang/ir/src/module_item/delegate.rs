use syn::Attribute;

use super::utils;

mod kw {
    syn::custom_keyword!(to);
}

#[derive(Debug, Clone)]
pub struct Delegate {
    pub stmts: Vec<DelegationStatement>
}

#[derive(Debug, Clone)]
pub struct DelegationStatement {
    pub delegate_to: syn::ExprField,
    pub delegation_block: DelegationBlock
}

#[derive(Debug, Clone)]
pub struct DelegationBlock {
    pub brace_token: syn::token::Brace,
    pub functions: Vec<DelegatedFunction>
}

#[derive(Debug, Clone)]
pub struct DelegatedFunction {
    pub attrs: Vec<Attribute>,
    pub ident: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>,
    pub ret: syn::ReturnType,
    pub full_sig: syn::Signature,
    pub visibility: syn::Visibility
}

impl syn::parse::Parse for Delegate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut stmts = vec![];
        while !input.is_empty() {
            stmts.push(input.parse::<DelegationStatement>()?);
        }
        Ok(Self { stmts })
    }
}

impl syn::parse::Parse for DelegationStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::to>()?;

        let delegate_to = input.parse::<syn::ExprField>()?;
        let delegation_block = input.parse::<DelegationBlock>()?;
        Ok(Self {
            delegate_to,
            delegation_block
        })
    }
}

impl syn::parse::Parse for DelegationBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let brace_token = syn::braced!(content in input);
        let mut functions = vec![];
        while !content.is_empty() {
            functions.push(content.parse::<DelegatedFunction>()?);
        }
        Ok(Self {
            brace_token,
            functions
        })
    }
}

impl syn::parse::Parse for DelegatedFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse::<syn::Visibility>()?;
        let fn_item = input.parse::<syn::TraitItemMethod>()?;

        let ident = fn_item.sig.ident.to_owned();
        let args = utils::extract_typed_inputs(&fn_item.sig);
        let ret = fn_item.clone().sig.output;
        let full_sig = fn_item.sig;

        Ok(Self {
            attrs,
            visibility,
            ident,
            args,
            ret,
            full_sig
        })
    }
}
