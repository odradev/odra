use syn::{parse_quote, spanned::Spanned};

pub fn ident_from_impl(impl_code: &syn::ItemImpl) -> Result<syn::Ident, syn::Error> {
    match &*impl_code.self_ty {
        syn::Type::Path(type_path) => {
            Ok(type_path.path.segments.last().expect("dupa").ident.clone())
        }
        ty => Err(syn::Error::new(
            ty.span(),
            "Only support impl for type path"
        ))
    }
}

pub fn function_arg_names(function: &syn::ImplItemFn) -> Vec<syn::Ident> {
    function
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                _ => None
            },
            _ => None
        })
        .collect()
}

pub fn function_args(function: &syn::ImplItemFn) -> Vec<syn::PatType> {
    function
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(pat_type.clone()),
            _ => None
        })
        .collect()
}

pub fn receiver_arg(function: &syn::ImplItemFn) -> Option<syn::Receiver> {
    function.sig.inputs.iter().find_map(|arg| match arg {
        syn::FnArg::Receiver(receiver) => Some(receiver.clone()),
        _ => None
    })
}

pub fn function_return_type(function: &syn::ImplItemFn) -> syn::ReturnType {
    function.sig.output.clone()
}

pub fn type_address() -> syn::Type {
    parse_quote!(odra2::types::Address)
}

pub fn type_contract_env() -> syn::Type {
    parse_quote!(odra2::ContractEnv)
}
