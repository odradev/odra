use derive_more::From;
use odra_ir::module::{Constructor, ImplItem, Method, ModuleImpl};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, TokenStreamExt, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, ReturnType, Type, TypePath};

use crate::GenerateCode;

#[derive(From)]
pub struct Deploy<'a> {
    contract: &'a ModuleImpl,
}

as_ref_for_contract_impl_generator!(Deploy);

impl GenerateCode for Deploy<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let struct_name = struct_ident.to_string();
        let ref_ident = format_ident!("{}Ref", struct_ident);

        let entrypoints = build_entrypoints(
            self.contract
                .methods()
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Method(method) => Some(method),
                    _ => None,
                }),
            struct_ident,
        );

        let constructors = build_constructors(
            self.contract
                .methods()
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Constructor(constructor) => Some(constructor),
                    _ => None,
                }),
            struct_ident,
        );

        let constructors_mock_vm = build_constructors_mock_vm(
            self.contract
                .methods()
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Constructor(constructor) => Some(constructor),
                    _ => None,
                }),
            entrypoints.clone(),
            constructors.clone(),
            struct_ident,
            ref_ident.clone(),
        );

        let constructors_wasm_test = build_constructors_wasm_test(
            self.contract
                .methods()
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Constructor(constructor) => Some(constructor),
                    _ => None,
                }),
            struct_ident,
            ref_ident.clone(),
        );

        let struct_snake_case = odra_utils::camel_to_snake(&struct_name);
        quote! {
            #[cfg(feature = "casper-test")]
            impl #struct_ident {
                pub fn deploy() -> #ref_ident {
                    let address = odra::test_env::register_contract(&#struct_snake_case, odra::types::CallArgs::new());
                    #ref_ident::at(address)
                }

                #constructors_wasm_test
            }

            #[cfg(feature = "mock-vm")]
            impl #struct_ident {

                pub fn deploy() -> #ref_ident {
                    use std::collections::HashMap;
                    use odra::types::{Bytes, CallArgs};

                    let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Option<Bytes>)>::new();
                    #entrypoints

                    let mut constructors = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Option<Bytes>)>::new();
                    #constructors

                    let address = odra::test_env::register_contract(None, constructors, entrypoints);
                    #ref_ident::at(address)
                }

                #constructors_mock_vm
            }
        }
    }
}

fn build_constructors_mock_vm<'a, C>(
    constructors: C,
    entrypoints_stream: TokenStream,
    constructors_stream: TokenStream,
    struct_ident: &Ident,
    ref_ident: Ident,
) -> TokenStream
where
    C: Iterator<Item = &'a Constructor>,
{
    constructors.map(|constructor| {
        let ty = Type::Path(TypePath { qself: None, path: From::from(ref_ident.clone()) });
        let deploy_fn_ident = format_ident!("deploy_{}", &constructor.ident);
        let sig = constructor.full_sig.clone();
        let constructor_ident = &constructor.ident;

        let inputs = sig.inputs.into_iter().filter(|i| match i {
            syn::FnArg::Receiver(_) => false,
            syn::FnArg::Typed(_) => true,
        }).collect::<Punctuated<_, _>>();

        let deploy_fn_sig = syn::Signature {
            ident: deploy_fn_ident,
            output: ReturnType::Type(Default::default(), Box::new(ty)),
            inputs,
            ..sig
        };

        let args = args_to_runtime_args_stream(&constructor.args);

        let fn_args = args_to_fn_args(&constructor.args);

        quote! {
            pub #deploy_fn_sig {
                use std::collections::HashMap;
                use odra::types::{Bytes, CallArgs};

                let mut entrypoints = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Option<Bytes>)>::new();
                #entrypoints_stream

                let mut constructors = HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Option<Bytes>)>::new();
                #constructors_stream

                let args = {
                    #args
                };
                let constructor: Option<(String, CallArgs, fn(String, CallArgs) -> Option<Bytes>)> = Some((
                    stringify!(#constructor_ident).to_string(),
                    args,
                    |name, args| {
                        let instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                        instance.#constructor_ident( #fn_args );
                        None
                    }
                ));
                let address = odra::test_env::register_contract(constructor, constructors, entrypoints);
                #ref_ident::at(address)
            }
        }
    }).collect::<TokenStream>()
}

fn build_constructors_wasm_test<'a, C>(
    constructors: C,
    struct_ident: &Ident,
    ref_ident: Ident,
) -> TokenStream
where
    C: Iterator<Item = &'a Constructor>,
{
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);

    constructors
        .map(|constructor| {
            let ty = Type::Path(TypePath {
                qself: None,
                path: From::from(ref_ident.clone()),
            });
            let deploy_fn_ident = format_ident!("deploy_{}", &constructor.ident);
            let sig = constructor.full_sig.clone();
            let constructor_ident = &constructor.ident;

            let inputs = sig
                .inputs
                .into_iter()
                .filter(|i| match i {
                    syn::FnArg::Receiver(_) => false,
                    syn::FnArg::Typed(_) => true,
                })
                .collect::<Punctuated<_, _>>();

            let deploy_fn_sig = syn::Signature {
                ident: deploy_fn_ident,
                output: ReturnType::Type(Default::default(), Box::new(ty)),
                inputs,
                ..sig
            };

            let args = args_to_runtime_args_stream(&constructor.args);

            quote! {
                pub #deploy_fn_sig {
                    use odra::types::CallArgs;
                    let mut args = { #args };
                    args.insert("constructor", stringify!(#constructor_ident)).unwrap();
                    let address = odra::test_env::register_contract(#struct_name_snake_case, &args);
                    #ref_ident::at(address)
                }
            }
        })
        .collect::<TokenStream>()
}

fn build_entrypoints<'a, T>(methods: T, struct_ident: &Ident) -> TokenStream
where
    T: Iterator<Item = &'a Method>,
{
    methods
        .map(|entrypoint| {
            let ident = &entrypoint.ident;
            let name = quote!(stringify!(#ident).to_string());
            let return_value = match &entrypoint.ret {
                ReturnType::Default => quote!(None),
                ReturnType::Type(_, _) => quote! {
                    let bytes = odra::types::ToBytes::to_bytes(&result).unwrap();
                    Some(odra::types::Bytes::from(bytes))
                },
            };
            let args = args_to_fn_args(&entrypoint.args);
            let arg_names = args_to_arg_names_stream(&entrypoint.args);
            let attached_value_check = match entrypoint.is_payable() {
                true => quote!(),
                false => quote! {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::odra_types::ExecutionError::non_payable());
                    }
                },
            };
            quote! {
                entrypoints.insert(#name, (#arg_names, |name, args| {
                    #attached_value_check
                    let instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                    let result = instance.#ident(#args);
                    #return_value
                }));
            }
        })
        .collect::<TokenStream>()
}

fn build_constructors<'a, T>(constructors: T, struct_ident: &Ident) -> TokenStream
where
    T: Iterator<Item = &'a Constructor>,
{
    constructors
        .map(|constructor| {
            let ident = &constructor.ident;
            let args = args_to_fn_args(&constructor.args);
            let arg_names = args_to_arg_names_stream(&constructor.args);

            quote! {
                constructors.insert(stringify!(#ident).to_string(), (#arg_names,
                    |name, args| {
                        let instance = <#struct_ident as odra::Instance>::instance(name.as_str());
                        instance.#ident( #args );
                        None
                    }
                ));
            }
        })
        .collect::<TokenStream>()
}

fn args_to_fn_args<'a, T>(args: T) -> Punctuated<TokenStream, Comma>
where
    T: IntoIterator<Item = &'a syn::PatType>,
{
    args.into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            quote!(args
                .get(stringify!(#pat))
                .cloned()
                .unwrap()
                .into_t()
                .unwrap())
        })
        .collect::<Punctuated<TokenStream, Comma>>()
}

fn args_to_runtime_args_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>,
{
    let mut tokens = quote!(let mut args = odra::types::CallArgs::new(););
    tokens.append_all(args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { args.insert(stringify!(#pat), #pat).unwrap(); }
    }));
    tokens.extend(quote!(args));
    tokens
}

fn args_to_arg_names_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>,
{
    let args_stream = args
        .into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            quote! { args.push(stringify!(#pat).to_string()); }
        })
        .collect::<TokenStream>();

    quote! {
        {
            let mut args: Vec<String> = vec![];
            #args_stream
            args
        }
    }
}
