use derive_more::From;
use odra_ir::module::{Constructor, Method, ModuleImpl};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma};

use crate::GenerateCode;

pub(super) mod common;
mod deployer_casper_livenet;
mod deployer_casper_test;
mod deployer_mock_vm;

#[derive(From)]
pub struct Deploy<'a> {
    contract: &'a ModuleImpl
}

as_ref_for_contract_impl_generator!(Deploy);

impl GenerateCode for Deploy<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let ref_ident = format_ident!("{}Ref", struct_ident);
        let deployer_ident = format_ident!("{}Deployer", struct_ident);
        let deployer_comment = format!("Deployer for the [{}] contract.", struct_ident);

        let method_defs: Vec<&Method> = self.contract.get_public_method_iter().collect();
        let constructor_defs: Vec<&Constructor> = self.contract.get_constructor_iter().collect();

        let mock_vm_deployer_impl = deployer_mock_vm::generate_code(
            struct_ident,
            &deployer_ident,
            &ref_ident,
            &constructor_defs,
            &method_defs
        );
        let casper_test_deployer_impl = deployer_casper_test::generate_code(
            struct_ident,
            &deployer_ident,
            &ref_ident,
            &constructor_defs
        );
        let casper_livenet_deployer_impl = deployer_casper_livenet::generate_code(
            struct_ident,
            &deployer_ident,
            &ref_ident,
            &constructor_defs,
            &method_defs
        );

        quote! {
            #[doc = #deployer_comment]
            pub struct #deployer_ident;

            #[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
            #casper_test_deployer_impl

            #[cfg(feature = "mock-vm")]
            #mock_vm_deployer_impl

            #[cfg(feature = "casper-livenet")]
            #casper_livenet_deployer_impl
        }
    }
}

fn args_to_fn_args<'a, T>(args: T) -> Punctuated<TokenStream, Comma>
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    args.into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            match &*arg.ty {
                syn::Type::Reference(ty) => match &*ty.elem {
                    ty if matches!(ty, syn::Type::Array(_) | syn::Type::Slice(_)) => {
                        quote!(&odra::types::UncheckedGetter::get::<
                            odra::prelude::vec::Vec<_>
                        >(args, stringify!(#pat)))
                    }
                    _ => quote!(&odra::types::UncheckedGetter::get(args, stringify!(#pat)))
                },
                ty if matches!(ty, syn::Type::Array(_) | syn::Type::Slice(_)) => {
                    quote!(&odra::types::UncheckedGetter::get::<
                        odra::prelude::vec::Vec<_>
                    >(args, stringify!(#pat)))
                }
                _ => quote!(odra::types::UncheckedGetter::get(args, stringify!(#pat)))
            }
        })
        .collect::<Punctuated<TokenStream, Comma>>()
}

fn args_to_fn_cl_values<'a, T>(args: T) -> Punctuated<TokenStream, Comma>
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    args.into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            match &*arg.ty {
                syn::Type::Reference(ty) => match &*ty.elem {
                    syn::Type::Array(inner_ty) => {
                        let inner_ty = &inner_ty.elem;
                        quote! {
                            &args.get(stringify!(#pat))
                                .cloned()
                                .unwrap()
                                .into_t::<odra::prelude::vec::Vec<#inner_ty>>()
                                .unwrap()
                                .as_slice()
                        }
                    }
                    syn::Type::Slice(inner_ty) => {
                        let inner_ty = &inner_ty.elem;
                        quote! {
                            &args.get(stringify!(#pat))
                                .cloned()
                                .unwrap()
                                .into_t::<odra::prelude::vec::Vec<#inner_ty>>()
                                .unwrap()
                                .as_slice()
                        }
                    }
                    ty => quote! {
                        &args.get(stringify!(#pat))
                            .cloned()
                            .unwrap()
                            .into_t::<#ty>()
                            .unwrap()
                    }
                },
                syn::Type::Array(inner_ty) => {
                    let inner_ty = &inner_ty.elem;
                    quote! {
                        args.get(stringify!(#pat))
                            .cloned()
                            .unwrap()
                            .into_t::<odra::prelude::vec::Vec<#inner_ty>>()
                            .unwrap()
                    }
                }
                syn::Type::Slice(inner_ty) => {
                    let inner_ty = &inner_ty.elem;
                    quote! {
                        args.get(stringify!(#pat))
                            .cloned()
                            .unwrap()
                            .into_t::<odra::prelude::vec::Vec<#inner_ty>>()
                            .unwrap()
                    }
                }
                ty => quote! {
                    args.get(stringify!(#pat))
                        .cloned()
                        .unwrap()
                        .into_t::<#ty>()
                        .unwrap()
                }
            }
        })
        .collect::<Punctuated<TokenStream, Comma>>()
}

fn args_to_runtime_args_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    let mut tokens = quote!(let mut args = odra::types::casper_types::RuntimeArgs::new(););
    tokens.append_all(args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { let _ = args.insert(stringify!(#pat), #pat.clone()); }
    }));
    tokens.extend(quote!(args));
    tokens
}

fn args_to_arg_names_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    let args_stream = args
        .into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            let pat = pat.to_token_stream().to_string();
            quote!(args.push(odra::prelude::string::String::from(#pat));)
        })
        .collect::<TokenStream>();

    quote! {
        {
            let mut args: odra::prelude::vec::Vec<odra::prelude::string::String> = odra::prelude::vec![];
            #args_stream
            args
        }
    }
}
