use odra_ir::module::{Constructor, Method};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::{args_to_arg_names_stream, args_to_fn_args2};

pub fn generate_code(
    struct_ident: &Ident,
    deployer_ident: &Ident,
    ref_ident: &Ident,
    _constructors: &[&Constructor],
    methods: &[&Method]
) -> TokenStream {
    // let entry = build_constructors(constructors, struct_ident, ref_ident);
    let entrypoint_calls = build_entrypoint_calls(methods, struct_ident);

    quote! {
        impl #deployer_ident {
            pub fn register(address: odra::types::Address) -> #ref_ident {
                use std::collections::HashMap;
                use odra::types::CallArgs;

                let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
                #entrypoint_calls

                odra::test_env::register_contract(address, entrypoints);
                #ref_ident::at(address)
            }
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
    let name = quote!(stringify!(#ident).to_string());
    let args = args_to_fn_args2(&entrypoint.args);
    let arg_names = args_to_arg_names_stream(&entrypoint.args);
    quote! {
        entrypoints.insert(#name, (#arg_names, |name, args| {
            let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
            let result = instance.#ident(#args);
            let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
            odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
        }));
    }
}
