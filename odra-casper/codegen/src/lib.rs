//! Set of functions to generate Casper contract.

use crate::call_method::CallMethod;

use self::wasm_entrypoint::WasmEntrypoint;
use odra_types::contract_def::ContractBlueprint;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, Path, Token};

mod arg;
mod call_method;
mod constructor;
mod entrypoints_def;
mod schema;
mod ty;
mod wasm_entrypoint;

pub use schema::gen_schema;

pub fn contract_ident() -> proc_macro2::Ident {
    proc_macro2::Ident::new("_contract", Span::call_site())
}

/// Given the ContractDef from Odra, generate Casper contract.
pub fn gen_contract(blueprint: ContractBlueprint) -> TokenStream2 {
    let keys = generate_storage_keys(&blueprint);
    let entrypoints = generate_entrypoints(&blueprint);
    let call_fn = generate_call(&blueprint);

    quote! {
        #![no_main]

        #keys

        #call_fn

        #entrypoints
    }
}

fn generate_storage_keys(blueprint: &ContractBlueprint) -> TokenStream2 {
    let keys_count = blueprint.keys_count as usize;
    let keys_literals = blueprint
        .keys
        .iter()
        .map(|k| quote!(#k))
        .collect::<Punctuated<TokenStream2, Token![,]>>();
    quote! {
        const KEYS: [&'static str; #keys_count] = [
            #keys_literals
        ];
    }
}

fn generate_entrypoints(blueprint: &ContractBlueprint) -> TokenStream2 {
    let path = fqn_to_path(blueprint.fqn);
    blueprint
        .entrypoints
        .iter()
        .flat_map(|ep| WasmEntrypoint(ep, &path).to_token_stream())
        .collect::<TokenStream2>()
}

fn generate_call(blueprint: &ContractBlueprint) -> TokenStream2 {
    let ref_fqn = blueprint.fqn.to_string() + "Ref";

    CallMethod::new(
        blueprint.events.to_vec(),
        blueprint.entrypoints.to_vec(),
        fqn_to_path(ref_fqn.as_str())
    )
    .to_token_stream()
}

fn fqn_to_path(fqn: &str) -> Path {
    syn::parse_str(fqn).expect("Invalid fqn")
}

#[cfg(test)]
fn assert_eq_tokens<A: ToTokens, B: ToTokens>(left: A, right: B) {
    let left = left.to_token_stream().to_string().replace(' ', "");
    let right = right.to_token_stream().to_string().replace(' ', "");
    pretty_assertions::assert_str_eq!(left, right);
}

#[macro_export]
macro_rules! gen_contract {
    ($contract:path, $name:literal) => {
        pub fn main() {
            let ident = <$contract as odra::types::contract_def::HasIdent>::ident();
            let entrypoints =
                <$contract as odra::types::contract_def::HasEntrypoints>::entrypoints();
            let events = <$contract as odra::types::contract_def::HasEvents>::events();
            for event in &events {
                if event.has_any() {
                    panic!("Event {} can't have Type::Any struct in it.", &event.ident);
                }
            }
            let keys = <$contract as odra::types::contract_def::Node>::keys();
            let keys_count = <$contract as odra::types::contract_def::Node>::count();

            let blueprint = odra::types::contract_def::ContractBlueprint {
                keys,
                keys_count,
                events: events.clone(),
                entrypoints: entrypoints.clone(),
                fqn: stringify!($contract)
            };
            let code = odra::casper::codegen::gen_contract(blueprint);

            let schema = odra::casper::codegen::gen_schema(&ident, &entrypoints, &events);

            use std::io::prelude::*;
            let mut source_file = std::fs::File::create(&format!("src/{}_wasm.rs", $name)).unwrap();
            source_file
                .write_all(&code.to_string().into_bytes())
                .unwrap();

            if !std::path::Path::new("../resources").exists() {
                std::fs::create_dir("../resources").unwrap();
            }

            let mut schema_file =
                std::fs::File::create(&format!("../resources/{}_schema.json", $name)).unwrap();
            schema_file.write_all(&schema.into_bytes()).unwrap();
        }
    };
}

#[cfg(test)]
mod tests {
    use odra_types::contract_def::{Argument, ContractBlueprint, Entrypoint, EntrypointType};
    use odra_types::Type;
    use quote::quote;

    use super::{assert_eq_tokens, gen_contract};

    #[test]
    fn test_contract_codegen() {
        let constructor = Entrypoint {
            ident: String::from("construct_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: Type::I32,
                is_ref: true
            }],
            ret: Type::Unit,
            ty: EntrypointType::Constructor {
                non_reentrant: false
            },
            is_mut: false
        };
        let entrypoint = Entrypoint {
            ident: String::from("call_me"),
            args: vec![],
            ret: Type::Bool,
            ty: EntrypointType::Public {
                non_reentrant: false
            },
            is_mut: false
        };

        let blueprint = ContractBlueprint {
            keys: vec!["key".to_string(), "a_b_c".to_string()],
            keys_count: 2,
            events: vec![],
            entrypoints: vec![constructor, entrypoint],
            fqn: "my_contract::MyContract"
        };
        let result = gen_contract(blueprint);

        assert_eq_tokens(
            result,
            quote! {
                #![no_main]

                const KEYS: [&'static str; 2usize] = [
                    "key",
                    "a_b_c"
                ];

                #[no_mangle]
                fn call() {
                    let schemas = vec![];
                    let mut entry_points = odra::casper::casper_types::EntryPoints::new();
                    entry_points.add_entry_point(odra::casper::casper_types::EntryPoint::new(
                        stringify!(construct_me),
                        {
                            let mut params: Vec<odra::casper::casper_types::Parameter> = Vec::new();
                            params.push(odra::casper::casper_types::Parameter::new(
                                stringify!(value),
                                odra::casper::casper_types::CLType::I32
                            ));
                            params
                        },
                        odra::casper::casper_types::CLType::Unit,
                        odra::casper::casper_types::EntryPointAccess::Groups(vec![
                            odra::casper::casper_types::Group::new("constructor_group")
                        ]),
                        odra::casper::casper_types::EntryPointType::Contract,
                    ));
                    entry_points.add_entry_point(odra::casper::casper_types::EntryPoint::new(
                        stringify!(call_me),
                        Vec::<odra::casper::casper_types::Parameter>::new(),
                        odra::casper::casper_types::CLType::Bool,
                        odra::casper::casper_types::EntryPointAccess::Public,
                        odra::casper::casper_types::EntryPointType::Contract,
                    ));
                    #[allow(dead_code)]
                    let contract_package_hash = odra::casper::utils::install_contract(entry_points, &schemas);
                    use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                    let constructor_access = odra::casper::utils::create_constructor_group(contract_package_hash);
                    let constructor_name = odra::casper::utils::load_constructor_name_arg();
                    match constructor_name.as_str() {
                        stringify!(construct_me) => {
                            let odra_address = odra::types::Address::try_from(contract_package_hash)
                                .map_err(|err| {
                                    let code = odra::types::ExecutionError::from(err).code();
                                    odra::casper::casper_types::ApiError::User(code)
                                })
                                .unwrap_or_revert();
                            let contract_ref = my_contract::MyContractRef::at(&odra_address);
                            let value = odra::casper::casper_contract::contract_api::runtime::get_named_arg(
                                stringify!(value)
                            );
                            contract_ref.construct_me(&value);
                        },
                        _ => odra::casper::utils::revert_on_unknown_constructor()
                    };
                    odra::casper::utils::revoke_access_to_constructor_group(
                        contract_package_hash,
                        constructor_access
                    );
                }
                #[no_mangle]
                fn construct_me() {
                    odra::casper::utils::assert_no_attached_value();
                    let (_contract, _): (my_contract::MyContract, _) = odra::StaticInstance::instance(&KEYS);
                    let value =
                        odra::casper::casper_contract::contract_api::runtime::get_named_arg(stringify!(value));
                    _contract.construct_me(&value);
                }
                #[no_mangle]
                fn call_me() {
                    odra::casper::utils::assert_no_attached_value();
                    let (_contract, _): (my_contract::MyContract, _) = odra::StaticInstance::instance(&KEYS);
                    use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                    let result = _contract.call_me();
                    let result = odra::casper::casper_types::CLValue::from_t(result).unwrap_or_revert();
                    odra::casper::casper_contract::contract_api::runtime::ret(result);
                }
            }
        );
    }
}
