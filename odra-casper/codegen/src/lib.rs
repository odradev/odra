//! Set of functions to generate Casper contract.

use crate::ty::CasperType;

use self::{
    constructor::WasmConstructor, entrypoints_def::ContractEntrypoints,
    wasm_entrypoint::WasmEntrypoint
};
use odra_types::contract_def::{Entrypoint, EntrypointType, Event};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Path, PathSegment, Token};

mod arg;
mod constructor;
mod entrypoints_def;
mod schema;
mod ty;
mod wasm_entrypoint;

pub use schema::gen_schema;

/// Given the ContractDef from Odra, generate Casper contract.
pub fn gen_contract(
    contract_ident: &str,
    contract_entrypoints: &[Entrypoint],
    events: &[Event],
    fqn: &str
) -> TokenStream2 {
    let entrypoints = generate_entrypoints(contract_entrypoints, fqn);
    let ref_fqn = fqn.to_string() + "Ref";
    let call_fn = generate_call(contract_ident, contract_entrypoints, events, &ref_fqn);

    quote! {
        #![no_main]

        use odra::Instance;

        #call_fn

        #entrypoints
    }
}

fn generate_entrypoints(entrypoints: &[Entrypoint], fqn: &str) -> TokenStream2 {
    let path = &fqn_to_path(fqn);
    entrypoints
        .iter()
        .flat_map(|ep| WasmEntrypoint(ep, path).to_token_stream())
        .collect::<TokenStream2>()
}

fn generate_call(
    contract_ident: &str,
    contract_entrypoints: &[Entrypoint],
    events: &[Event],
    ref_fqn: &str
) -> TokenStream2 {
    let entrypoints = ContractEntrypoints(contract_entrypoints);
    let contract_def_name_snake = odra_utils::camel_to_snake(contract_ident);
    let package_hash = format!("{}_package_hash", contract_def_name_snake);

    let constructors = contract_entrypoints
        .iter()
        .filter(|ep| matches!(ep.ty, EntrypointType::Constructor { .. }))
        .collect::<Vec<_>>();

    let ref_path = &fqn_to_path(ref_fqn);
    let call_constructor = WasmConstructor(constructors, ref_path);

    let events_schema = events
        .iter()
        .map(|ev| {
            let ident = &ev.ident;

            let fields = ev
                .args
                .iter()
                .map(|arg| {
                    let field = &arg.ident;
                    let ty = CasperType(&arg.ty);
                    quote!((#field, #ty))
                })
                .collect::<Punctuated<TokenStream2, Comma>>();

            quote! {
                odra::casper::utils::build_event(
                    #ident,
                    vec![#fields]
                )
            }
        })
        .collect::<Punctuated<TokenStream2, Comma>>();

    quote! {
        #[no_mangle]
        fn call() {
            let (contract_package_hash, _) = odra::casper::casper_contract::contract_api::storage::create_contract_package_at_hash();
            odra::casper::casper_contract::contract_api::runtime::put_key(#package_hash, contract_package_hash.into());

            #entrypoints

            let schemas = vec![
                #events_schema
            ];
            odra::casper::utils::add_contract_version(
                contract_package_hash,
                entry_points,
                schemas
            );

            #call_constructor
        }
    }
}

fn fqn_to_path(fqn: &str) -> Path {
    let paths = fqn.split("::").collect::<Vec<_>>();

    let segments = Punctuated::<PathSegment, Token![::]>::from_iter(
        paths
            .iter()
            .map(|ident| PathSegment::from(format_ident!("{}", ident)))
    );

    syn::Path {
        leading_colon: None,
        segments
    }
}

#[cfg(test)]
fn assert_eq_tokens<A: ToTokens, B: ToTokens>(left: A, right: B) {
    let left = left.to_token_stream().to_string();
    let right = right.to_token_stream().to_string();
    pretty_assertions::assert_str_eq!(left, right);
}

#[cfg(test)]
mod tests {
    use odra_types::contract_def::{Argument, Entrypoint, EntrypointType};
    use odra_types::Type;
    use quote::{quote, ToTokens};

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

        let path: syn::Path = syn::parse_str("my_contract::MyContract").unwrap();

        let fqn = path.to_token_stream().to_string().replace(' ', "");

        let contract_ident = String::from("MyContract");
        let entrypoints = vec![constructor, entrypoint];
        let events = vec![];

        let result = gen_contract(&contract_ident, &entrypoints, &events, &fqn);

        assert_eq_tokens(
            result,
            quote! {
                #![no_main]

                use odra::Instance;
                #[no_mangle]
                fn call() {
                    let (contract_package_hash , _) = odra::casper::casper_contract::contract_api::storage::create_contract_package_at_hash();
                    odra::casper::casper_contract::contract_api::runtime::put_key(
                        "my_contract_package_hash",
                        contract_package_hash.into()
                    );

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
                            odra::casper::casper_types::Group::new("constructor")
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
                    let schemas = vec![];
                    odra::casper::utils::add_contract_version(
                        contract_package_hash,
                        entry_points,
                        schemas
                    );

                    if odra::casper::utils::named_arg_exists("constructor") {
                        use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                        let constructor_access: odra::casper::casper_types::URef =
                            odra::casper::casper_contract::contract_api::storage::create_contract_user_group(
                                contract_package_hash,
                                "constructor",
                                1,
                                Default::default()
                            )
                            .unwrap_or_revert()
                            .pop()
                            .unwrap_or_revert();
                        let constructor_name = odra::casper::casper_contract::contract_api::runtime::get_named_arg::<
                            String
                        >("constructor");
                        let constructor_name = constructor_name.as_str();
                        match constructor_name {
                            stringify!(construct_me) => {
                                let odra_address = odra::types::Address::try_from(contract_package_hash)
                                    .map_err(|err| {
                                        let code = odra::types::ExecutionError::from(err).code();
                                        odra::casper::casper_types::ApiError::User(code)
                                    })
                                    .unwrap_or_revert();
                                let mut contract_ref = my_contract::MyContractRef::at(odra_address);
                                let value = odra::casper::casper_contract::contract_api::runtime::get_named_arg(
                                    stringify!(value)
                                );
                                contract_ref.construct_me(&value);
                            },
                            _ => {}
                        };
                        let mut urefs = std::collections::BTreeSet::new();
                        urefs.insert(constructor_access);
                        odra::casper::casper_contract::contract_api::storage::remove_contract_user_group_urefs(
                            contract_package_hash,
                            "constructor",
                            urefs
                        )
                        .unwrap_or_revert();
                    }
                }

                #[no_mangle]
                fn construct_me() {
                    odra::casper::utils::assert_no_attached_value();
                    let contract = my_contract::MyContract::instance("contract");
                    let value =
                        odra::casper::casper_contract::contract_api::runtime::get_named_arg(stringify!(value));
                    contract.construct_me(&value);
                }
                #[no_mangle]
                fn call_me() {
                    odra::casper::utils::assert_no_attached_value();
                    let contract = my_contract::MyContract::instance("contract");
                    use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                    let result = contract.call_me();
                    let result = odra::casper::casper_types::CLValue::from_t(result).unwrap_or_revert();
                    odra::casper::casper_contract::contract_api::runtime::ret(result);
                }
            }
        );
    }
}

#[macro_export]
macro_rules! gen_contract {
    ($contract:path, $name:literal) => {
        fn main() {
            let ident = <$contract as odra::types::contract_def::HasIdent>::ident();
            let entrypoints =
                <$contract as odra::types::contract_def::HasEntrypoints>::entrypoints();
            let events = <$contract as odra::types::contract_def::HasEvents>::events();
            let code = odra::casper::codegen::gen_contract(
                &ident,
                &entrypoints,
                &events,
                stringify!($contract)
            );

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
