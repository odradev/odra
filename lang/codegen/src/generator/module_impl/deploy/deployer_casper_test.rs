use odra_ir::module::Constructor;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, ReturnType, Type, TypePath};

use super::args_to_runtime_args_stream;

pub fn generate_code(
    struct_ident: &Ident,
    deployer_ident: &Ident,
    ref_ident: &Ident,
    constructors: &[&Constructor]
) -> TokenStream {
    let constructors = build_constructors(constructors, struct_ident, ref_ident);

    quote! {
        impl #deployer_ident {
            #constructors
        }
    }
}

fn build_constructors(
    constructors: &[&Constructor],
    struct_ident: &Ident,
    ref_ident: &Ident
) -> TokenStream {
    if constructors.is_empty() {
        build_default_constructor(struct_ident, ref_ident)
    } else {
        constructors
            .iter()
            .map(|constructor| build_constructor(constructor, struct_ident, ref_ident))
            .collect::<TokenStream>()
    }
}

fn build_default_constructor(struct_ident: &Ident, ref_ident: &Ident) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);

    quote! {
        pub fn default() -> #ref_ident {
            let address = odra::test_env::register_contract(&#struct_name_snake_case, odra::types::CallArgs::new());
            #ref_ident::at(address)
        }
    }
}

fn build_constructor(
    constructor: &Constructor,
    struct_ident: &Ident,
    ref_ident: &Ident
) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);
    let ty = Type::Path(TypePath {
        qself: None,
        path: From::from(ref_ident.clone())
    });
    let sig = constructor.full_sig.clone();
    let constructor_ident = &constructor.ident;

    let inputs = sig
        .inputs
        .into_iter()
        .filter(|i| match i {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(_) => true
        })
        .collect::<Punctuated<_, _>>();

    let fn_sig = syn::Signature {
        output: ReturnType::Type(Default::default(), Box::new(ty)),
        inputs,
        ..sig
    };

    let args = args_to_runtime_args_stream(&constructor.args);

    quote! {
        pub #fn_sig {
            let mut args = { #args };
            args.insert("constructor", stringify!(#constructor_ident));
            let address = odra::test_env::register_contract(#struct_name_snake_case, args);
            #ref_ident::at(address)
        }
    }
}