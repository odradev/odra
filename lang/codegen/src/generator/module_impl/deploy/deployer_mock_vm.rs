use odra_ir::module::{Constructor, Method};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, ReturnType, Type, TypePath};

use super::{args_to_arg_names_stream, args_to_fn_args, args_to_runtime_args_stream};

pub fn generate_code(
    struct_ident: &Ident,
    deployer_ident: &Ident,
    ref_ident: &Ident,
    constructors: &[&Constructor],
    methods: &[&Method]
) -> TokenStream {
    let entrypoint_calls = build_entrypoints_calls(methods, struct_ident);
    let constructor_calls = build_constructor_calls(constructors, struct_ident);
    let constructors = build_constructors(
        constructors,
        entrypoint_calls,
        constructor_calls,
        struct_ident,
        ref_ident
    );

    quote! {
        impl #deployer_ident {
            #constructors
        }
    }
}

fn build_constructors(
    constructors: &[&Constructor],
    entrypoints_stream: TokenStream,
    constructors_stream: TokenStream,
    struct_ident: &Ident,
    ref_ident: &Ident
) -> TokenStream {
    if constructors.is_empty() {
        build_default_constructor(entrypoints_stream, constructors_stream, ref_ident)
    } else {
        constructors
            .iter()
            .map(|constructor| {
                build_constructor(
                    constructor,
                    entrypoints_stream.clone(),
                    constructors_stream.clone(),
                    struct_ident,
                    ref_ident
                )
            })
            .collect::<TokenStream>()
    }
}

fn build_constructor(
    constructor: &Constructor,
    entrypoints_stream: TokenStream,
    constructors_stream: TokenStream,
    struct_ident: &Ident,
    ref_ident: &Ident
) -> TokenStream {
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
    let fn_args = args_to_fn_args(&constructor.args);

    quote! {
        pub #fn_sig {
            use std::collections::HashMap;
            use odra::types::{CallArgs};

            let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
            #entrypoints_stream

            let mut constructors = HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
            #constructors_stream

            let args = {
                #args
            };
            let constructor: Option<(String, &CallArgs, fn(String, &CallArgs) -> Vec<u8>)> = Some((
                stringify!(#constructor_ident).to_string(),
                &args,
                |name, args| {
                    let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                    instance.#constructor_ident( #fn_args );
                    Vec::new()
                }
            ));
            let address = odra::test_env::register_contract(constructor, constructors, entrypoints);
            #ref_ident::at(address)
        }
    }
}

fn build_default_constructor(
    entrypoints_stream: TokenStream,
    constructors_stream: TokenStream,
    ref_ident: &Ident
) -> TokenStream {
    quote! {
        pub fn default() -> #ref_ident {
            use std::collections::HashMap;
            use odra::types::CallArgs;

            let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
            #entrypoints_stream

            let mut constructors = HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
            #constructors_stream

            let address = odra::test_env::register_contract(None, constructors, entrypoints);
            #ref_ident::at(address)
        }
    }
}

fn build_entrypoints_calls(methods: &[&Method], struct_ident: &Ident) -> TokenStream {
    methods
        .iter()
        .map(|entrypoint| {
            let ident = &entrypoint.ident;
            let name = quote!(stringify!(#ident).to_string());
            let return_value = match &entrypoint.ret {
                ReturnType::Default => quote!(Vec::new()),
                ReturnType::Type(_, _) => quote! {
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            };
            let args = args_to_fn_args(&entrypoint.args);
            let arg_names = args_to_arg_names_stream(&entrypoint.args);
            let attached_value_check = match entrypoint.is_payable() {
                true => quote!(),
                false => quote! {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                }
            };
            let non_reentrant = entrypoint.attrs.iter().any(|a| a.is_non_reentrant());
            let reentrancy_check = match non_reentrant {
                true => quote! {
                    if odra::contract_env::get_var("__reentrancy_guard").unwrap_or_default(){
                        odra::contract_env::revert(odra::types::ExecutionError::reentrant_call());
                    }
                    odra::contract_env::set_var("__reentrancy_guard", true);
                },
                false => quote!()
            };
            let reentrancy_cleanup = match non_reentrant {
                true => quote!(odra::contract_env::set_var("__reentrancy_guard", false);),
                false => quote!()
            };
            quote! {
                entrypoints.insert(#name, (#arg_names, |name, args| {
                    #reentrancy_check
                    #attached_value_check
                    let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                    let result = instance.#ident(#args);
                    #reentrancy_cleanup
                    #return_value
                }));
            }
        })
        .collect::<TokenStream>()
}

fn build_constructor_calls(constructors: &[&Constructor], struct_ident: &Ident) -> TokenStream {
    constructors
        .iter()
        .map(|constructor| {
            let ident = &constructor.ident;
            let args = args_to_fn_args(&constructor.args);
            let arg_names = args_to_arg_names_stream(&constructor.args);

            quote! {
                constructors.insert(stringify!(#ident).to_string(), (#arg_names,
                    |name, args| {
                        let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                        instance.#ident( #args );
                        Vec::new()
                    }
                ));
            }
        })
        .collect::<TokenStream>()
}
