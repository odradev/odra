use crate::utils::misc::AsExpr;
use syn::parse_quote;

pub fn new_module(
    contract_ident: &syn::Ident,
    module_ident: &syn::Ident,
    env_rc_ident: &syn::Ident
) -> syn::Stmt {
    let module_ty = super::ty::module();
    parse_quote!(
        let #contract_ident = <#module_ident as #module_ty>::new(#env_rc_ident);
    )
}

pub fn new_mut_module(
    contract_ident: &syn::Ident,
    module_ident: &syn::Ident,
    env_rc_ident: &syn::Ident
) -> syn::Stmt {
    let module_ty = super::ty::module();
    parse_quote!(
        let mut #contract_ident = <#module_ident as #module_ty>::new(#env_rc_ident);
    )
}

pub fn get_named_arg(arg_ident: &syn::Ident, env_ident: &syn::Ident, ty: &syn::Type) -> syn::Stmt {
    let arg_name = arg_ident.to_string();
    parse_quote!(let #arg_ident = #env_ident.get_named_arg::<#ty>(#arg_name);)
}

pub fn new_execution_env(ident: &syn::Ident, env_rc_ident: &syn::Ident) -> syn::Stmt {
    parse_quote!(let #ident = odra::ExecutionEnv::new(#env_rc_ident.clone());)
}

pub fn new_rc(var_ident: &syn::Ident, env_ident: &syn::Ident) -> syn::Stmt {
    parse_quote!(let #var_ident = Rc::new(#env_ident);)
}

pub fn new_mut_vec_with_capacity<E: AsExpr>(ident: &syn::Ident, capacity_expr: &E) -> syn::Stmt {
    let ty = super::ty::vec();
    let expr = capacity_expr.as_expr();
    parse_quote!(let mut #ident = #ty::with_capacity(#expr);)
}
