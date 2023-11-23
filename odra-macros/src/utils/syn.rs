use syn::{parse_quote, spanned::Spanned};

pub fn ident_from_impl(impl_code: &syn::ItemImpl) -> Result<syn::Ident, syn::Error> {
    type_to_ident(&impl_code.self_ty)
}

pub fn ident_from_struct(struct_code: &syn::ItemStruct) -> syn::Ident {
    struct_code.ident.clone()
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

pub fn struct_fields_ident(item: &syn::ItemStruct) -> Result<Vec<syn::Ident>, syn::Error> {
    if let syn::Fields::Named(named) = &item.fields {
        let err_msg = "Invalid field. Module fields must be named";
        named
            .named
            .iter()
            .map(|f| f.ident.clone().ok_or(syn::Error::new(f.span(), err_msg)))
            .collect::<Result<Vec<syn::Ident>, syn::Error>>()
    } else {
        Err(syn::Error::new_spanned(
            &item.fields,
            "Invalid fields. Module fields must be named"
        ))
    }
}

pub fn struct_fields(item: &syn::ItemStruct) -> Result<Vec<(syn::Ident, syn::Type)>, syn::Error> {
    if let syn::Fields::Named(named) = &item.fields {
        let err_msg = "Invalid field. Module fields must be named";
        named
            .named
            .iter()
            .map(|f| {
                f.ident.clone()
                    .ok_or(syn::Error::new_spanned(f, err_msg))
                    .map(|i| (i, f.ty.clone()))
            })
            .collect()
    } else {
        Err(syn::Error::new_spanned(
            &item.fields,
            "Invalid fields. Module fields must be named"
        ))
    }
}

pub fn visibility_pub() -> syn::Visibility {
    parse_quote!(pub)
}

pub fn type_to_ident(ty: &syn::Type) -> Result<syn::Ident, syn::Error>{
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.clone())
            .ok_or(syn::Error::new(type_path.span(), "Invalid type path")),
        ty => Err(syn::Error::new(
            ty.span(),
            "Only support impl for type path"
        ))
    }
}

pub fn clear_generics(ty: &syn::Type) -> Result<syn::Type, syn::Error> {
    match ty {
        syn::Type::Path(type_path) => clear_path(type_path).map(|p| syn::Type::Path(p)),
        ty => Err(syn::Error::new(
            ty.span(),
            "Only support impl for type path"
        ))
    }
}

fn clear_path(ty: &syn::TypePath) -> Result<syn::TypePath, syn::Error> {
    let mut owned_ty = ty.to_owned();

    let mut segment = owned_ty.path
        .segments
        .last_mut()
        .ok_or(syn::Error::new(ty.span(), "Invalid type path"))?;
    segment.arguments = syn::PathArguments::None;

    Ok(owned_ty)
}