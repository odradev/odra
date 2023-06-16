use odra_ir::module::{Constructor, Method};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::generator::module_impl::deploy::common;

use super::{args_to_arg_names_stream, args_to_fn_cl_values, args_to_runtime_args_stream};

pub fn generate_code(
    struct_ident: &Ident,
    deployer_ident: &Ident,
    ref_ident: &Ident,
    constructors: &[&Constructor],
    methods: &[&Method]
) -> TokenStream {
    let entrypoint_calls = build_entrypoint_calls(methods, struct_ident);
    let constructors = build_constructors(constructors, &entrypoint_calls, struct_ident, ref_ident);

    quote! {
        impl #deployer_ident {
            pub fn register(address: odra::types::Address) -> #ref_ident {
                use odra::types::CallArgs;

                let mut entrypoints = alloc::collections::BTreeMap::<
                    alloc::string::String,
                    (alloc::vec::Vec<alloc::string::String>, fn(alloc::string::String, &CallArgs) -> alloc::vec::Vec<u8>)
                >::new();
                #entrypoint_calls

                odra::client_env::register_existing_contract(address, entrypoints);
                #ref_ident::at(&address)
            }

            #constructors
        }

    }
}

fn build_entrypoint_calls(methods: &[&Method], struct_ident: &Ident) -> TokenStream {
    methods
        .iter()
        .map(|entrypoint| build_entrypoint_call(entrypoint, struct_ident))
        .collect::<TokenStream>()
}

fn build_entrypoint_call(entrypoint: &Method, struct_ident: &Ident) -> TokenStream {
    let ident = &entrypoint.ident;
    let name = quote!(alloc::string::String::from(stringify!(#ident)));
    let args = args_to_fn_cl_values(&entrypoint.args);
    let arg_names = args_to_arg_names_stream(&entrypoint.args);
    quote! {
        entrypoints.insert(#name, (#arg_names, |name, args| {
            let keys = <#struct_ident as odra::types::contract_def::Node>::__keys();
            let keys = keys
                .iter()
                .map(alloc::string::String::as_str)
                .collect::<alloc::vec::Vec<_>>();
            let (mut instance, _) = <#struct_ident as odra::StaticInstance>::instance(keys.as_slice());
            let result = instance.#ident(#args);
            let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
            odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
        }));
    }
}

fn build_constructors(
    constructors: &[&Constructor],
    entrypoint_calls: &TokenStream,
    struct_ident: &Ident,
    ref_ident: &Ident
) -> TokenStream {
    if constructors.is_empty() {
        build_default_constructor(struct_ident, ref_ident, entrypoint_calls)
    } else {
        constructors
            .iter()
            .map(|constructor| {
                build_constructor(constructor, struct_ident, ref_ident, entrypoint_calls)
            })
            .collect::<TokenStream>()
    }
}

fn build_default_constructor(
    struct_ident: &Ident,
    ref_ident: &Ident,
    entrypoint_calls: &TokenStream
) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);

    quote! {
        pub fn default() -> #ref_ident {
            use odra::types::CallArgs;
            let mut entrypoints = alloc::collections::BTreeMap::<alloc::string::String, (alloc::vec::Vec<alloc::string::String>, fn(alloc::string::String, &CallArgs) -> alloc::vec::Vec<u8>)>::new();
            #entrypoint_calls

            let address = odra::client_env::deploy_new_contract(&#struct_name_snake_case, odra::types::CallArgs::new(), entrypoints, None);
            #ref_ident::at(&address)
        }
    }
}

fn build_constructor(
    constructor: &Constructor,
    struct_ident: &Ident,
    ref_ident: &Ident,
    entrypoint_calls: &TokenStream
) -> TokenStream {
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);

    let constructor_ident = &constructor.ident;

    let fn_sig = common::constructor_sig(constructor, ref_ident);

    let args = args_to_runtime_args_stream(&constructor.args);

    quote! {
        pub #fn_sig {
            use odra::types::CallArgs;

            let mut entrypoints = alloc::collections::BTreeMap::<alloc::string::String, (alloc::vec::Vec<alloc::string::String>, fn(alloc::string::String, &CallArgs) -> alloc::vec::Vec<u8>)>::new();
            #entrypoint_calls

            let args = { #args };

            let constructor = alloc::string::String::from(stringify!(#constructor_ident));
            let address = odra::client_env::deploy_new_contract(#struct_name_snake_case, args, entrypoints, Some(constructor));
            #ref_ident::at(&address)
        }
    }
}
