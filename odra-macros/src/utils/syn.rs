use proc_macro2::TokenStream;
use syn::{parse_quote, spanned::Spanned, DeriveInput};

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
            }) => Some(pat.ident.clone()),
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

pub fn docs_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect()
}

pub fn string_docs(attrs: &[syn::Attribute]) -> Vec<String> {
    let attrs = docs_attrs(attrs);

    let mut docs = Vec::new();
    for attr in attrs {
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(str),
                ..
            }) = &nv.value
            {
                docs.push(str.value());
            }
        }
    }
    docs
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

pub fn extract_named_field(input: &DeriveInput) -> syn::Result<Vec<syn::Field>> {
    if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
            .iter()
            .map(|f| {
                if f.ident.is_none() {
                    Err(syn::Error::new(f.span(), "Unnamed field"))
                } else {
                    Ok(f.clone())
                }
            })
            .collect()
    } else {
        Ok(vec![])
    }
}

pub fn extract_unit_variants(input: &DeriveInput) -> syn::Result<Vec<syn::Variant>> {
    if let syn::Data::Enum(syn::DataEnum { variants, .. }) = &input.data {
        let is_valid = variants
            .iter()
            .all(|v| matches!(v.fields, syn::Fields::Unit));
        if is_valid {
            Ok(variants.into_iter().cloned().collect())
        } else {
            Err(syn::Error::new_spanned(
                variants,
                "Expected a unit enum variant."
            ))
        }
    } else {
        Ok(vec![])
    }
}

pub fn transform_variants<F: Fn(String, u16, Vec<String>) -> TokenStream>(
    variants: &[syn::Variant],
    f: F
) -> TokenStream {
    let mut discriminant = 0u16;
    let variants = variants.iter().map(|v| {
        let docs = string_docs(&v.attrs);
        let name = v.ident.to_string();
        if let Some((_, syn::Expr::Lit(lit))) = &v.discriminant {
            if let syn::Lit::Int(int) = &lit.lit {
                discriminant = int.base10_parse().unwrap();
            }
        };
        let result = f(name, discriminant, docs);
        discriminant += 1;
        result
    });

    quote::quote!(odra::prelude::vec![#(#variants)*])
}
