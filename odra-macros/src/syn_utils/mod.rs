pub fn ident_from_impl(impl_code: &syn::ItemImpl) -> syn::Ident {
    match &*impl_code.self_ty {
        syn::Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("Only support impl for type path"),
    }
}

pub fn function_arg_names(function: &syn::ImplItemFn) -> Vec<syn::Ident> {
    function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                _ => panic!("Only support function arg as ident"),
            },
            _ => panic!("Only support function arg as ident"),
        })
        .collect()
}

pub fn function_args(function: &syn::ImplItemFn) -> Vec<syn::PatType> {
    function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => pat_type.clone(),
            _ => panic!("Only support function arg as ident"),
        })
        .collect()
}

pub fn function_return_type(function: &syn::ImplItemFn) -> syn::Type {
    match &function.sig.output {
        syn::ReturnType::Type(_, ty) => *ty.clone(),
        _ => panic!("Only support function with return type"),
    }
}