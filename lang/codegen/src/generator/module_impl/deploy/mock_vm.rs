use odra_ir::module::{Constructor, ImplItem, Method, ModuleImpl};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, ReturnType, Type, TypePath};

use crate::generator::module_impl::deploy::{
    args_to_arg_names_stream, args_to_fn_args, args_to_runtime_args_stream
};

pub fn build_constructors(contract: &ModuleImpl) -> TokenStream {
    let struct_ident = contract.ident();
    let ref_ident = format_ident!("{}Ref", struct_ident);

    let entrypoints = build_entrypoints(
        contract.methods().iter().filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None
        }),
        struct_ident
    );

    let constructors = _build_constructors(
        contract.methods().iter().filter_map(|item| match item {
            ImplItem::Constructor(constructor) => Some(constructor),
            _ => None
        }),
        struct_ident
    );

    let mut constructors_mock_vm = build_constructors_mock_vm(
        contract.methods().iter().filter_map(|item| match item {
            ImplItem::Constructor(constructor) => Some(constructor),
            _ => None
        }),
        entrypoints.clone(),
        constructors.clone(),
        struct_ident,
        ref_ident.clone()
    );

    if constructors_mock_vm.is_empty() {
        constructors_mock_vm = quote! {
            pub fn default() -> #ref_ident {
                use std::collections::HashMap;
                use odra::types::CallArgs;

                let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
                #entrypoints

                let mut constructors = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
                #constructors

                let address = odra::test_env::register_contract(None, constructors, entrypoints);
                #ref_ident::at(address)
            }
        };
    }

    constructors_mock_vm
}

fn build_constructors_mock_vm<'a, C>(
    constructors: C,
    entrypoints_stream: TokenStream,
    constructors_stream: TokenStream,
    struct_ident: &Ident,
    ref_ident: Ident
) -> TokenStream
where
    C: Iterator<Item = &'a Constructor>
{
    constructors.map(|constructor| {
        let ty = Type::Path(TypePath { qself: None, path: From::from(ref_ident.clone()) });
        let sig = constructor.full_sig.clone();
        let constructor_ident = &constructor.ident;

        let inputs = sig.inputs.into_iter().filter(|i| match i {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(_) => true,
        }).collect::<Punctuated<_, _>>();

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
                use odra::types::CallArgs;

                let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
                #entrypoints_stream

                let mut constructors = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
                #constructors_stream

                let args = {
                    #args
                };
                let constructor: Option<(String, CallArgs, fn(String, CallArgs) -> Vec<u8>)> = Some((
                    stringify!(#constructor_ident).to_string(),
                    args,
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
    }).collect::<TokenStream>()
}

fn build_entrypoints<'a, T>(methods: T, struct_ident: &Ident) -> TokenStream
where
    T: Iterator<Item = &'a Method>
{
    methods
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
                false => quote!(),
            };
            let reentrancy_cleanup = match non_reentrant {
                true => quote!(odra::contract_env::set_var("__reentrancy_guard", false);),
                false => quote!(),
            };
            quote! {
                entrypoints.insert(
                    #name,
                    (
                        #arg_names,
                        |name, args| {
                            #reentrancy_check
                            #attached_value_check
                            let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                            let result = instance.#ident(#args);
                            #reentrancy_cleanup
                            #return_value
                        }
                    )
                );
            }
        })
        .collect::<TokenStream>()
}

fn _build_constructors<'a, T>(constructors: T, struct_ident: &Ident) -> TokenStream
where
    T: Iterator<Item = &'a Constructor>
{
    constructors
        .map(|constructor| {
            let ident = &constructor.ident;
            let args = args_to_fn_args(&constructor.args);
            let arg_names = args_to_arg_names_stream(&constructor.args);
            quote! {
                constructors.insert(
                    stringify!(#ident).to_string(),
                    (
                        #arg_names,
                        |name, args| {
                            let mut instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                            instance.#ident( #args );
                            Vec::new()
                        }
                    )
                );
            }
        })
        .collect::<TokenStream>()
}
