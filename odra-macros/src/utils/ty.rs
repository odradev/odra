use syn::parse_quote;

fn slice_to_vec(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Slice(syn::TypeSlice { elem, .. }) => {
            parse_quote!(odra::prelude::vec::Vec<#elem>)
        }
        _ => ty.clone()
    }
}

/// The type an argument is passed and stored as: references are dropped and a slice becomes a
/// `Vec`, so `&[Address]` is read from the call as `Vec<Address>`.
pub fn unreferenced_ty(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(syn::TypeReference { elem, .. }) => {
            if matches!(**elem, syn::Type::Reference(_)) {
                unreferenced_ty(elem)
            } else {
                slice_to_vec(elem)
            }
        }
        _ => slice_to_vec(ty)
    }
}
