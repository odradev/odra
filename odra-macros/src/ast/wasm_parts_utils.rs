use crate::{ir::FnIR, utils};
use syn::parse_quote;

pub fn param_parameters(func: &FnIR) -> syn::Expr {
    let params = func
        .named_args()
        .iter()
        .map(|arg| arg.name_and_ty())
        .filter_map(|result| match result {
            Ok(data) => Some(data),
            Err(_) => None
        })
        .map(|(name, ty)| utils::expr::new_parameter(name, ty))
        .collect::<Vec<_>>();
    parse_quote!(vec![#(#params),*])
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

pub fn insert_arg_stmt(ident: &syn::Ident) -> syn::Stmt {
    let name = ident.to_string();
    let args = utils::ident::named_args();

    syn::parse_quote!(let _ = #args.insert(
        #name, 
        odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_named_arg(#name)
    );)
}
