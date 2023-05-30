use derive_more::From;
use odra_ir::module::{Constructor, Method, ModuleImpl};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
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
            quote!(&args.get(stringify!(#pat)))
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
    T: IntoIterator<Item = &'a syn::PatType>
{
    let mut tokens = quote!(let mut args = odra::types::CallArgs::new(););
    tokens.append_all(args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { args.insert(stringify!(#pat), #pat.clone()); }
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
