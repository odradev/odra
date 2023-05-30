use odra_ir::module::Constructor;
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, ReturnType, Type, TypePath};

pub fn constructor_sig(constructor: &Constructor, ref_ident: &Ident) -> syn::Signature {
    let ty = Type::Path(TypePath {
        qself: None,
        path: From::from(ref_ident.clone())
    });
    let sig = constructor.full_sig.clone();

    let inputs = sig
        .inputs
        .into_iter()
        .filter(|i| matches!(i, syn::FnArg::Typed(_)))
        .collect::<Punctuated<_, _>>();

    syn::Signature {
        output: ReturnType::Type(Default::default(), Box::new(ty)),
        inputs,
        ..sig
    }
}
