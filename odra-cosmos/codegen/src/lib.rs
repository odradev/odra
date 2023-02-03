//! Set of functions to generate Casper contract.

use std::str::FromStr;

use convert_case::{Case, Casing};
use odra_types::contract_def::{ContractDef, Entrypoint, EntrypointType};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::Path;

const UNIT: &str = "()";

/// Given the ContractDef from Odra, generate Casper contract.
pub fn gen_contract(contract_def: ContractDef, fqn: String) -> TokenStream2 {
    let init_filter = |ep: &&Entrypoint| ep.ty == EntrypointType::Constructor;
    let exec_filter = |ep: &&Entrypoint| ep.ty != EntrypointType::Constructor && &ep.ret == UNIT;
    let query_filter = |ep: &&Entrypoint| ep.ty != EntrypointType::Constructor && &ep.ret != UNIT;

    let init_variants = build_message_variants(&contract_def, init_filter);
    let exec_variants = build_message_variants(&contract_def, exec_filter);
    let query_variants = build_message_variants(&contract_def, query_filter);

    let ident_init_message = format_ident!("InitMessage");
    let ident_exec_message = format_ident!("ExecMessage");
    let ident_query_message = format_ident!("QueryMessage");

    let contract_path = fqn_to_path(&fqn);
    let message_ident = format_ident!("message");

    let init_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        &ident_init_message,
        &message_ident,
        init_filter,
        to_variant_branch
    );
    let exec_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        &ident_exec_message,
        &message_ident,
        exec_filter,
        to_variant_branch
    );
    let query_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        &ident_query_message,
        &message_ident,
        query_filter,
        to_query_variant_branch
    );

    let init_msg = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #ident_init_message {
            #init_variants
        }
    };
    let exec_msg = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #ident_exec_message {
            #exec_variants
        }
    };
    let query_msg = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #ident_query_message {
            #query_variants
        }
    };

    let parse_message = parse_message();
    let parse_query_message = parse_query_message();

    quote! {
        #![no_main]
        use odra::{types::Address, Instance};

        #[no_mangle]
        fn instantiate(ptr0: u32, ptr1: u32, ptr2: u32) -> u32 {
            odra_cosmos_backend::instantiate(&init_fn, ptr0, ptr1, ptr2)
        }

        #[no_mangle]
        fn execute(ptr0: u32, ptr1: u32, ptr2: u32) -> u32 {
            odra_cosmos_backend::execute(&exe_fn, ptr0, ptr1, ptr2)
        }

        #[no_mangle]
        fn query(ptr0: u32, ptr1: u32) -> u32 {
            odra_cosmos_backend::query(&query_fn, ptr0, ptr1)
        }

        #init_msg
        #exec_msg
        #query_msg

        fn init_fn(input: &[u8]) -> Result<odra_cosmos_backend::cosmwasm_std::Response, String> {
            let #message_ident: #ident_init_message = #parse_message
            #init_variant_matching
            Ok(odra_cosmos_backend::cosmwasm_std::Response::new())
        }

        fn exe_fn(input: &[u8]) -> Result<odra_cosmos_backend::cosmwasm_std::Response, String> {
            let #message_ident: #ident_exec_message = #parse_message
            #exec_variant_matching
            Ok(odra_cosmos_backend::cosmwasm_std::Response::new())
        }

        fn query_fn(input: &[u8]) -> odra_cosmos_backend::cosmwasm_std::StdResult<odra_cosmos_backend::cosmwasm_std::Binary> {
            let #message_ident: #ident_query_message = #parse_query_message
            #query_variant_matching
        }

    }
}

fn to_variant_branch(ep: &Entrypoint, contract_path: &Path, message_ty: &Ident) -> TokenStream2 {
    let variant_ident = format_ident!("{}", ep.ident.to_case(Case::Pascal));
    let fn_ident = format_ident!("{}", ep.ident);
    let args = ep
        .args
        .iter()
        .map(|arg| {
            let ident = format_ident!("{}", arg.ident);
            quote!(#ident,)
        })
        .collect::<TokenStream2>();
    let contract_instance = match ep.is_mut {
        true => quote!(let mut contract = #contract_path::instance("contract");),
        false => quote!(let contract = #contract_path::instance("contract");)
    };
    quote! {
        #message_ty::#variant_ident { #args } => {
            #contract_instance
            contract.#fn_ident(#args);
        }
    }
}

fn to_query_variant_branch(
    ep: &Entrypoint,
    contract_path: &Path,
    message_ty: &Ident
) -> TokenStream2 {
    let variant_ident = format_ident!("{}", ep.ident.to_case(Case::Pascal));
    let fn_ident = format_ident!("{}", ep.ident);
    let args = ep
        .args
        .iter()
        .map(|arg| {
            let ident = format_ident!("{}", arg.ident);
            quote!(#ident,)
        })
        .collect::<TokenStream2>();
    let contract_instance = match ep.is_mut {
        true => quote!(let mut contract = #contract_path::instance("contract");),
        false => quote!(let contract = #contract_path::instance("contract");)
    };
    quote! {
        #message_ty::#variant_ident { #args } => {
            #contract_instance
            let result = contract.#fn_ident(#args);
            odra_cosmos_backend::cosmwasm_std::to_binary(&result)
        }
    }
}

fn to_variant_token_stream(ep: &Entrypoint) -> TokenStream2 {
    let ident = format_ident!("{}", ep.ident.to_case(Case::Pascal));
    let args = ep
        .args
        .iter()
        .map(|arg| {
            let ident = format_ident!("{}", arg.ident);
            let ty = TokenStream2::from_str(&arg.ty)
                .expect("An argument type should be a valid TokenStream");
            let ty = syn::parse2::<syn::Type>(ty).expect("Should be a valid type");
            quote!(#ident: #ty,)
        })
        .collect::<TokenStream2>();
    quote!(#ident { #args },)
}

fn build_message_variants<'a, F>(def: &ContractDef, f: F) -> TokenStream2
where
    F: FnMut(&&Entrypoint) -> bool
{
    def.entrypoints
        .iter()
        .filter(f)
        .map(to_variant_token_stream)
        .collect::<TokenStream2>()
}

fn build_variant_matching<'a, F, M>(
    def: &ContractDef,
    contract_path: &Path,
    message_ty: &Ident,
    message_ident: &Ident,
    f: F,
    mut m: M
) -> TokenStream2
where
    F: FnMut(&&Entrypoint) -> bool,
    M: FnMut(&Entrypoint, &Path, &Ident) -> TokenStream2
{
    let matching = def
        .entrypoints
        .iter()
        .filter(f)
        .map(|ep| m(ep, contract_path, message_ty))
        .collect::<TokenStream2>();

    if matching.is_empty() {
        return matching;
    }

    quote! {
        match #message_ident {
            #matching
        }
    }
}

fn parse_message() -> TokenStream2 {
    quote! {
        match odra_cosmos_backend::cosmwasm_std::from_slice(input) {
            Ok(val) => val,
            Err(err) => {
                return Err(err.to_string())
            }
        };
    }
}

fn parse_query_message() -> TokenStream2 {
    quote! {
        match odra_cosmos_backend::cosmwasm_std::from_slice(input) {
            Ok(val) => val,
            Err(err) => {
                return Err(err)
            }
        };
    }
}

fn fqn_to_path(fqn: &String) -> Path {
    let tokens = TokenStream2::from_str(fqn).expect("fqn should be a valid token stream");
    syn::parse2::<syn::Path>(tokens).expect("Couldn't parse token stream")
}

#[cfg(test)]
mod test {
    use crate::fqn_to_path;

    #[test]
    fn parsing_fqn() {
        let fqn = String::from("full::contract::path::Contract");

        let path: syn::Path = syn::parse_quote! {
            full::contract::path::Contract
        };
        assert_eq!(path, fqn_to_path(&fqn));
    }

    #[test]
    fn parsing_fqn_with_leading_colons() {
        let fqn = String::from("::full::contract::path::Contract");

        let path: syn::Path = syn::parse_quote! {
            ::full::contract::path::Contract
        };
        assert_eq!(path, fqn_to_path(&fqn));
    }
}
