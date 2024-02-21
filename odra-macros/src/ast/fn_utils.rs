use crate::{
    ir::{FnArgIR, FnIR},
    utils
};
use syn::token::Pub;
use syn::{parse_quote, Attribute, FnArg};

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
    comment: Option<Attribute>,
    vis_token: Option<syn::Token![pub]>,
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
            vis_token: None,
            comment: None,
            fn_token: Default::default(),
            fn_name: name.clone(),
            paren_token: Default::default(),
            args: syn::punctuated::Punctuated::from_iter(args),
            ret_ty,
            block
        }
    }

    pub fn public(mut self, comment: String) -> FnItem {
        let comment = format!(" {}", comment);
        self.comment = Some(parse_quote!(#[doc = #comment]));
        self.vis_token = Some(Pub::default());
        self
    }

    pub fn instanced(mut self) -> FnItem {
        self.args.insert(0, parse_quote!(&self));
        self
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

    pub fn _make_pub(mut self, comment: String) -> Self {
        self.fn_item = self.fn_item.public(comment);
        self
    }

    pub fn _make_instanced(mut self) -> Self {
        self.fn_item = self.fn_item.instanced();
        self
    }
}
