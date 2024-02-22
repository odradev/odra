use proc_macro2::TokenStream;
use syn::{parse_quote, spanned::Spanned};

pub fn ident_from_impl(impl_code: &syn::ItemImpl) -> syn::Result<syn::Ident> {
    last_segment_ident(&impl_code.self_ty)
}

pub fn ident_from_struct(struct_code: &syn::ItemStruct) -> syn::Ident {
    struct_code.ident.clone()
}

pub fn function_name(sig: &syn::Signature) -> String {
    sig.ident.to_string()
}

pub fn function_arg_names(sig: &syn::Signature) -> Vec<syn::Ident> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(syn::PatType {
                pat: box syn::Pat::Ident(pat),
                ..
            }) => {
                Some(pat.ident.clone())
            },
            _ => None
        })
        .collect()
}

pub fn function_named_args(sig: &syn::Signature) -> Vec<&syn::FnArg> {
    sig.inputs
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

pub fn function_typed_args(sig: &syn::Signature) -> Vec<syn::PatType> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(pat_type.clone()),
            _ => None
        })
        .collect()
}

pub fn receiver_arg(sig: &syn::Signature) -> Option<syn::Receiver> {
    sig.inputs.iter().find_map(|arg| match arg {
        syn::FnArg::Receiver(receiver) => Some(receiver.clone()),
        _ => None
    })
}

pub fn function_return_type(sig: &syn::Signature) -> syn::ReturnType {
    sig.output.clone()
}

pub fn struct_fields_ident(item: &syn::ItemStruct) -> syn::Result<Vec<syn::Ident>> {
    map_fields(item, |f| {
        f.ident.clone().ok_or(syn::Error::new(
            f.span(),
            "Invalid field. Module fields must be named"
        ))
    })
}

pub fn struct_typed_fields(item: &syn::ItemStruct) -> syn::Result<Vec<(syn::Ident, syn::Type)>> {
    map_fields(item, |f| {
        f.ident
            .clone()
            .ok_or(syn::Error::new_spanned(
                f,
                "Invalid field. Module fields must be named"
            ))
            .map(|i| (i, f.ty.clone()))
    })
}

fn map_fields<T, F: FnMut(&syn::Field) -> syn::Result<T>>(
    item: &syn::ItemStruct,
    f: F
) -> syn::Result<Vec<T>> {
    if item.fields.is_empty() {
        return Ok(vec![]);
    }
    if let syn::Fields::Named(named) = &item.fields {
        named.named.iter().map(f).collect()
    } else {
        Err(syn::Error::new_spanned(
            &item.fields,
            "Invalid fields. Module fields must be named"
        ))
    }
}

pub fn derive_item_variants(item: &syn::DeriveInput) -> syn::Result<Vec<syn::Ident>> {
    match &item.data {
        syn::Data::Struct(syn::DataStruct { fields, .. }) => fields
            .iter()
            .map(|f| {
                f.ident
                    .clone()
                    .ok_or(syn::Error::new(f.span(), "Unnamed field"))
            })
            .collect::<Result<Vec<_>, _>>(),
        syn::Data::Enum(syn::DataEnum { variants, .. }) => {
            let is_valid = variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit));
            if is_valid {
                Ok(variants.iter().map(|v| v.ident.clone()).collect::<Vec<_>>())
            } else {
                Err(syn::Error::new_spanned(
                    variants,
                    "Expected a unit enum variant."
                ))
            }
        }
        _ => Err(syn::Error::new_spanned(
            item,
            "Struct with named fields expected"
        ))
    }
}

pub fn visibility_pub() -> syn::Visibility {
    parse_quote!(pub)
}

pub fn visibility_default() -> syn::Visibility {
    parse_quote!()
}

pub fn last_segment_ident(ty: &syn::Type) -> syn::Result<syn::Ident> {
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

// A path like <Option::<U256> as odra::casper_types::CLTyped>
pub fn as_casted_ty_stream(ty: &syn::Type, as_ty: syn::Type) -> TokenStream {
    let ty = match ty {
        syn::Type::Path(type_path) => {
            let mut segments: syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> =
                type_path.path.segments.clone();
            // the syntax <Option<U256> as odra::casper_types::CLTyped>::cl_type() is invalid
            // it should be <Option::<U256> as odra::casper_types::CLTyped>::cl_type()
            if let Some(ps) = segments.first_mut() {
                if let syn::PathArguments::AngleBracketed(ab) = &ps.arguments {
                    let generic_arg: syn::AngleBracketedGenericArguments = parse_quote!(::#ab);
                    ps.arguments = syn::PathArguments::AngleBracketed(generic_arg);
                }
            }
            syn::Type::Path(syn::TypePath {
                path: syn::Path {
                    leading_colon: None,
                    segments
                },
                ..type_path.clone()
            })
        }
        _ => ty.clone()
    };

    parse_quote!(<#ty as #as_ty>)
}

pub fn is_ref(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}
