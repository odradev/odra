use crate::ir::{FnArgIR, FnIR};

pub fn insert_args_stmts<F: FnMut(&FnArgIR) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> Vec<syn::Stmt> {
    fun.named_args()
        .iter()
        .map(insert_arg_fn)
        .collect::<Vec<_>>()
}
