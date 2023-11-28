use crate::{ir::FnIR, utils};

pub fn runtime_args_block<F: FnMut(&syn::Ident) -> syn::Stmt>(
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

pub fn insert_args_stmts<F: FnMut(&syn::Ident) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> Vec<syn::Stmt> {
    fun.arg_names()
        .iter()
        .map(insert_arg_fn)
        .collect::<Vec<_>>()
}
