use odra_ir::module::{Constructor, Method};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::ReturnType;

use crate::generator::module_impl::deploy::common;

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
    let constructor_ident = &constructor.ident;

    let fn_sig = common::constructor_sig(constructor, ref_ident);

    let args = args_to_runtime_args_stream(&constructor.args);
    let fn_args = args_to_fn_args(&constructor.args);

    quote! {
        pub #fn_sig {
            use odra::types::casper_types::RuntimeArgs;

            let mut entrypoints = odra::prelude::collections::BTreeMap::<
                odra::prelude::string::String,
                (odra::prelude::vec::Vec<odra::prelude::string::String>, fn(odra::prelude::string::String, &RuntimeArgs) -> odra::prelude::vec::Vec<u8>)
            >::new();
            #entrypoints_stream

            let mut constructors = odra::prelude::collections::BTreeMap::<
                odra::prelude::string::String,
                (odra::prelude::vec::Vec<odra::prelude::string::String>, fn(odra::prelude::string::String, &RuntimeArgs) -> odra::prelude::vec::Vec<u8>)
            >::new();
            #constructors_stream

            let args = {
                #args
            };
            let constructor: Option<(
                odra::prelude::string::String, &RuntimeArgs,
                fn(odra::prelude::string::String, &RuntimeArgs) -> odra::prelude::vec::Vec<u8>)
            > = Some((
                odra::prelude::string::String::from(stringify!(#constructor_ident)),
                &args,
                |name, args| {
                    let keys = <#struct_ident as odra::types::contract_def::Node>::__keys();
                    let keys = keys
                        .iter()
                        .map(odra::prelude::string::String::as_str)
                        .collect::<odra::prelude::vec::Vec<_>>();
                    let (mut instance, _) = <#struct_ident as odra::StaticInstance>::instance(keys.as_slice());
                    instance.#constructor_ident( #fn_args );
                    odra::prelude::vec::Vec::new()
                }
            ));
            let address = odra::test_env::register_contract(constructor, constructors, entrypoints);
            #ref_ident::at(&address)
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
            use odra::types::casper_types::RuntimeArgs;

            let mut entrypoints = odra::prelude::collections::BTreeMap::<
                odra::prelude::string::String,
                (odra::prelude::vec::Vec<odra::prelude::string::String>, fn(odra::prelude::string::String, &RuntimeArgs) -> odra::prelude::vec::Vec<u8>)
            >::new();
            #entrypoints_stream

            let mut constructors = odra::prelude::collections::BTreeMap::<
                odra::prelude::string::String,
                (odra::prelude::vec::Vec<odra::prelude::string::String>, fn(odra::prelude::string::String, &RuntimeArgs) -> odra::prelude::vec::Vec<u8>)
            >::new();
            #constructors_stream

            let address = odra::test_env::register_contract(None, constructors, entrypoints);
            #ref_ident::at(&address)
        }
    }
}

fn build_entrypoints_calls(methods: &[&Method], struct_ident: &Ident) -> TokenStream {
    methods
        .iter()
        .map(|entrypoint| {
            let ident = &entrypoint.ident;
            let name = quote!(odra::prelude::string::String::from(stringify!(#ident)));
            let arg_names = args_to_arg_names_stream(&entrypoint.args);
            let return_value = return_value(entrypoint);
            let args = args_to_fn_args(&entrypoint.args);
            let attached_value_check = attached_value(entrypoint);
            let (reentrancy_check, reentrancy_cleanup) = reentrancy_code(entrypoint);

            quote! {
                entrypoints.insert(#name, (#arg_names, |name, args| {
                    #reentrancy_check
                    #attached_value_check
                    let keys = <#struct_ident as odra::types::contract_def::Node>::__keys();
                    let keys = keys
                        .iter()
                        .map(odra::prelude::string::String::as_str)
                        .collect::<odra::prelude::vec::Vec<_>>();
                    let (mut instance, _) = <#struct_ident as odra::StaticInstance>::instance(keys.as_slice());
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
                constructors.insert(odra::prelude::string::String::from(stringify!(#ident)), (#arg_names,
                    |name, args| {
                        let keys = <#struct_ident as odra::types::contract_def::Node>::__keys();
                        let keys = keys
                            .iter()
                            .map(odra::prelude::string::String::as_str)
                            .collect::<odra::prelude::vec::Vec<_>>();
                        let (mut instance, _) = <#struct_ident as odra::StaticInstance>::instance(keys.as_slice());
                        instance.#ident( #args );
                        odra::prelude::vec::Vec::new()
                    }
                ));
            }
        })
        .collect::<TokenStream>()
}

fn reentrancy_code(entrypoint: &Method) -> (Option<TokenStream>, Option<TokenStream>) {
    let non_reentrant = entrypoint.attrs.iter().any(|a| a.is_non_reentrant());
    let reentrancy_check = non_reentrant.then(|| {
        quote! {
            if odra::contract_env::get_var(b"__reentrancy_guard").unwrap_or_default(){
                odra::contract_env::revert(odra::types::ExecutionError::reentrant_call());
            }
            odra::contract_env::set_var(b"__reentrancy_guard", true);
        }
    });
    let reentrancy_cleanup =
        non_reentrant.then(|| quote!(odra::contract_env::set_var(b"__reentrancy_guard", false);));
    (reentrancy_check, reentrancy_cleanup)
}

fn attached_value(entrypoint: &Method) -> TokenStream {
    match entrypoint.is_payable() {
        true => quote!(),
        false => quote! {
            if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                odra::contract_env::revert(odra::types::ExecutionError::non_payable());
            }
        }
    }
}

fn return_value(entrypoint: &Method) -> TokenStream {
    match &entrypoint.ret {
        ReturnType::Default => quote!(odra::prelude::vec::Vec::new()),
        ReturnType::Type(_, _) => quote!(odra::types::MockSerializable::ser(&result).unwrap())
    }
}
