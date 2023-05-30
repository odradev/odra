use syn::{PatType, FnArg};

pub fn is_mut(sig: &syn::Signature) -> bool {
    let receiver = sig.inputs.iter().find_map(|input| match input {
        syn::FnArg::Receiver(receiver) => Some(receiver),
        syn::FnArg::Typed(_) => None
    });
    receiver.and_then(|r| r.mutability).is_some()
}

pub fn is_ref(ty: &PatType) -> bool {
    matches!(&*ty.ty, syn::Type::Reference(_))
}

pub fn ty<'a>(ty: &'a PatType) -> &'a syn::Type {
    deref_ty(&*ty.ty)
}

fn deref_ty<'a>(ty: &'a syn::Type) -> &'a syn::Type { 
    match ty {
        syn::Type::Reference(r) => deref_ty(&r.elem),
        other => other
    }
}

pub fn typed_arg(arg: &FnArg) -> Option<PatType> {
    match arg {
        syn::FnArg::Receiver(_) => None,
        syn::FnArg::Typed(pat) => Some(pat.clone())
    }
}

pub fn extract_typed_inputs(sig: &syn::Signature) -> syn::punctuated::Punctuated<syn::PatType, syn::token::Comma> {
    sig
        .inputs
        .iter()
        .filter_map(typed_arg)
        .collect::<syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>>()
}