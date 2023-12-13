use crate::{
    ir::{FnArgIR, FnIR},
    utils
};
use syn::{parse_quote, FnArg};

pub fn runtime_args_block<F: FnMut(&FnArgIR) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> syn::Block {
    let runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let insert_args = insert_args_stmts(fun, insert_arg_fn);

    syn::parse_quote!({
        let mut #args = #runtime_args;
        #(#insert_args)*
        #args
    })
}

pub fn insert_args_stmts<F: FnMut(&FnArgIR) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> Vec<syn::Stmt> {
    fun.named_args()
        .iter()
        .map(insert_arg_fn)
        .collect::<Vec<_>>()
}

#[derive(syn_derive::ToTokens)]
pub struct FnItem {
    fn_token: syn::Token![fn],
    fn_name: syn::Ident,
    #[syn(parenthesized)]
    paren_token: syn::token::Paren,
    #[syn(in = paren_token)]
    args: syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    ret_ty: syn::ReturnType,
    block: syn::Block
}

impl FnItem {
    pub fn new(
        name: &syn::Ident,
        args: Vec<syn::FnArg>,
        ret_ty: syn::ReturnType,
        block: syn::Block
    ) -> Self {
        Self {
            fn_token: Default::default(),
            fn_name: name.clone(),
            paren_token: Default::default(),
            args: syn::punctuated::Punctuated::from_iter(args),
            ret_ty,
            block
        }
    }
}

#[derive(syn_derive::ToTokens)]
pub struct SingleArgFnItem {
    fn_item: FnItem
}

impl SingleArgFnItem {
    pub fn new(name: &syn::Ident, arg: FnArg, ret_ty: syn::ReturnType, block: syn::Block) -> Self {
        Self {
            fn_item: FnItem::new(name, vec![arg], ret_ty, block)
        }
    }
}

#[derive(syn_derive::ToTokens)]
pub struct SelfFnItem {
    fn_item: SingleArgFnItem
}

impl SelfFnItem {
    pub fn new(name: &syn::Ident, ret_ty: syn::ReturnType, block: syn::Block) -> Self {
        let self_ty = utils::ty::self_ref();
        Self {
            fn_item: SingleArgFnItem::new(name, parse_quote!(#self_ty), ret_ty, block)
        }
    }
}
