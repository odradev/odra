use crate::{
    ir::{FnArgIR, FnIR},
    utils
};
use syn::parse_quote;

pub fn param_parameters(func: &FnIR) -> syn::Expr {
    let params = func
        .named_args()
        .iter()
        .map(|arg| arg.name_and_ty())
        .filter_map(Result::ok)
        .map(|(name, ty)| utils::expr::new_parameter(name, ty))
        .collect::<Vec<_>>();
    if params.is_empty() {
        parse_quote!(vec![])
    } else {
        parse_quote!(vec![#(#params),*].into_iter().filter_map(|x| x).collect())
    }
}

pub fn param_access(func: &FnIR) -> syn::Expr {
    match func.is_constructor() {
        true => utils::expr::entry_point_group("constructor_group"),
        false => utils::expr::entry_point_public()
    }
}

pub fn param_ret_ty(func: &FnIR) -> syn::Expr {
    match func.return_type() {
        syn::ReturnType::Default => utils::expr::unit_cl_type(),
        syn::ReturnType::Type(_, ty) => utils::expr::as_cl_type(&ty)
    }
}

pub fn insert_arg_stmt(arg: &FnArgIR) -> syn::Stmt {
    let (name, ty) = arg.name_and_ty().unwrap();
    let args = utils::ident::named_args();
    syn::parse_quote!(odra::args::EntrypointArgument::insert_runtime_arg(
        exec_env.get_named_arg::<#ty>(#name),
        #name,
        &mut #args
    );)
}
