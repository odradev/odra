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

    let contract_path = fqn_to_path(&fqn);
    let action_ident = format_ident!("action");
    let action_ty_ident = format_ident!("Action");

    let init_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        init_filter,
        to_variant_branch
    );
    let exec_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        exec_filter,
        to_variant_branch
    );
    let query_variant_matching = build_variant_matching(
        &contract_def,
        &contract_path,
        query_filter,
        to_query_variant_branch
    );

    let parse_message = parse_message();
    let parse_query_message = parse_query_message();

    quote! {
        #![no_main]
        use odra::UnwrapOrRevert;
        use odra::{types::Address, Instance};

        #[no_mangle]
        fn instantiate(ptr0: u32, ptr1: u32, ptr2: u32) -> u32 {
            odra::cosmos::instantiate(&init_fn, ptr0, ptr1, ptr2)
        }

        #[no_mangle]
        fn execute(ptr0: u32, ptr1: u32, ptr2: u32) -> u32 {
            odra::cosmos::execute(&exe_fn, ptr0, ptr1, ptr2)
        }

        #[no_mangle]
        fn query(ptr0: u32, ptr1: u32) -> u32 {
            odra::cosmos::query(&query_fn, ptr0, ptr1)
        }

        fn init_fn(input: &[u8]) -> Result<odra::cosmos::cosmwasm_std::Response, String> {
            let #action_ident: #action_ty_ident = #parse_message
            match action.name.as_str() {
                #init_variant_matching
                _ => return Err(String::from("Unknown action"))
            }
            Ok(odra::cosmos::cosmwasm_std::Response::new())
        }

        fn exe_fn(input: &[u8]) -> Result<odra::cosmos::cosmwasm_std::Response, String> {
            let #action_ident: #action_ty_ident = #parse_message
            match action.name.as_str() {
                #exec_variant_matching
                _ => return Err(String::from("Unknown action"))
            }
            Ok(odra::cosmos::cosmwasm_std::Response::new())
        }

        fn query_fn(input: &[u8]) -> odra::cosmos::cosmwasm_std::StdResult<odra::cosmos::cosmwasm_std::Binary> {
            let #action_ident: #action_ty_ident = #parse_query_message

            match action.name.as_str() {
                #query_variant_matching
                _ => return Err(odra::cosmos::cosmwasm_std::StdError::NotFound { kind: String::from("Unknown command")})
            }
        }

        #[derive(Debug, PartialEq)]
        struct Action {
            pub name: String,
            pub args: Vec<Vec<u8>>,
        }

        impl odra::cosmos::serde::Serialize for Action {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: odra::cosmos::serde::Serializer {

                let mut s = serializer.serialize_struct("Action", 2)?;
                odra::cosmos::serde::ser::SerializeStruct::serialize_field(&mut s, "name", &self.name)?;
                odra::cosmos::serde::ser::SerializeStruct::serialize_field(&mut s, "args", &self.args)?;
                odra::cosmos::serde::ser::SerializeStruct::end(s)
            }
        }
        struct ActionVisitor;

        impl<'de> odra::cosmos::serde::de::Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an Action")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                where
                    A: odra::cosmos::serde::de::MapAccess<'de>, {
                    let mut name: Option<String> = None;
                    let mut args: Option<Vec<Vec<u8>>> = None;
                    while let Some(key) =
                        match odra::cosmos::serde::de::MapAccess::next_key::<String>(&mut map) {
                            Ok(val) => val,
                            Err(err) => return Err(err)
                        }
                    {
                        match key.as_str() {
                            "name" => {
                                name = Some(
                                    match odra::cosmos::serde::de::MapAccess::next_value::<String>(&mut map) {
                                        Ok(val) => val,
                                        Err(err) => return Err(err)
                                    },
                                );
                            },
                            "args" => {
                                args = Some(
                                    match odra::cosmos::serde::de::MapAccess::next_value::<Vec<Vec<u8>>>(&mut map) {
                                        Ok(val) => val,
                                        Err(err) => return Err(err)
                                    },
                                );
                            },
                            _ => odra::contract_env::revert(Error::UnknownAction),
                        }
                    }
                    Ok(Action { name: name.unwrap(), args: args.unwrap() })
            }
        }

        impl <'de> odra::cosmos::serde::Deserialize<'de> for Action {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: odra::cosmos::serde::Deserializer<'de> {
                    deserializer.deserialize_struct(
                        "Action",
                        &["name", "args"],
                        ActionVisitor
                    )
            }
        }

        fn get_arg<T: odra::types::OdraType>(bytes: Vec<u8>) -> T {
            T::deser(bytes).unwrap()
        }

        odra::execution_error! {
            enum Error {
                UnknownAction => 1000
            }
        }
    }
}

fn to_variant_branch(ep: &Entrypoint, contract_path: &Path) -> TokenStream2 {
    let fn_ident = format_ident!("{}", ep.ident);
    let get_args = get_args(ep);
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
        stringify!(#fn_ident) => {
            #contract_instance
            #get_args
            contract.#fn_ident(#args);
        }
    }
}

fn to_query_variant_branch(ep: &Entrypoint, contract_path: &Path) -> TokenStream2 {
    let fn_ident = format_ident!("{}", ep.ident);
    let get_args = get_args(ep);
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
        stringify!(#fn_ident) => {
            #contract_instance
            #get_args
            let result = contract.#fn_ident(#args);
            odra::cosmos::cosmwasm_std::to_binary(&result)
        }
    }
}

fn get_args(ep: &Entrypoint) -> TokenStream2 {
    ep.args
        .iter()
        .enumerate()
        .map(|(idx, arg)| {
            let ident = format_ident!("{}", arg.ident);
            quote!(let #ident = get_arg(action.args.get(#idx).unwrap_or_revert().clone());)
        })
        .collect::<TokenStream2>()
}

fn build_variant_matching<'a, F, M>(
    def: &ContractDef,
    contract_path: &Path,
    f: F,
    mut m: M
) -> TokenStream2
where
    F: FnMut(&&Entrypoint) -> bool,
    M: FnMut(&Entrypoint, &Path) -> TokenStream2
{
    def.entrypoints
        .iter()
        .filter(f)
        .map(|ep| m(ep, contract_path))
        .collect::<TokenStream2>()
}

fn parse_message() -> TokenStream2 {
    quote! {
        match odra::cosmos::cosmwasm_std::from_slice(input) {
            Ok(val) => val,
            Err(err) => {
                return Err(err.to_string())
            }
        };
    }
}

fn parse_query_message() -> TokenStream2 {
    quote! {
        match odra::cosmos::cosmwasm_std::from_slice(input) {
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
