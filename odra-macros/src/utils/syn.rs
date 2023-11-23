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
            syn::FnArg::Typed(syn::PatType {
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Some(pat.ident.clone()),
            _ => None
        })
        .collect()
}

pub fn function_named_args(function: &syn::ImplItemFn) -> Vec<&syn::FnArg> {
    function
        .sig
        .inputs
        .iter()
        .filter(|arg| {
            matches!(
                arg,
                syn::FnArg::Typed(syn::PatType {
                    pat: box syn::Pat::Ident(_),
                    ..
                })
            )
        })
        .collect::<Vec<_>>()
}

pub fn function_typed_args(function: &syn::ImplItemFn) -> Vec<syn::PatType> {
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

pub fn visibility_pub() -> syn::Visibility {
    parse_quote!(pub)
}
