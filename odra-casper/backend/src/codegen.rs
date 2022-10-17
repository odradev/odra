//! Set of functions to generate Casper contract.

use self::{
    constructor::WasmConstructor, entrypoints_def::ContractEntrypoints,
    wasm_entrypoint::WasmEntrypoint,
};
use odra::contract_def::{ContractDef, EntrypointType};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{punctuated::Punctuated, Path, PathSegment, Token};

mod arg;
mod constructor;
mod entrypoints_def;
mod ty;
mod wasm_entrypoint;

/// Given the ContractDef from Odra, generate Casper contract.
pub fn gen_contract(contract_def: ContractDef, fqn: String) -> TokenStream2 {
    let entrypoints = generate_entrypoints(&contract_def, fqn.clone());
    let call_fn = generate_call(&contract_def, fqn + "Ref");

    quote! {
        #![no_main]

        use odra::Instance;
        use odra_casper_backend as casper_backend;

        #call_fn

        #entrypoints
    }
}

fn generate_entrypoints(contract_def: &ContractDef, fqn: String) -> TokenStream2 {
    let path = &fqn_to_path(fqn);
    contract_def
        .entrypoints
        .iter()
        .flat_map(|ep| WasmEntrypoint(ep, path).to_token_stream())
        .collect::<TokenStream2>()
}

fn generate_call(contract_def: &ContractDef, ref_fqn: String) -> TokenStream2 {
    let entrypoints = ContractEntrypoints(&contract_def.entrypoints);
    let contract_def_name_snake = odra::utils::camel_to_snake(&contract_def.ident);
    let package_hash = format!("{}_package_hash", contract_def_name_snake);

    let constructors = contract_def
        .entrypoints
        .iter()
        .filter(|ep| ep.ty == EntrypointType::Constructor)
        .collect::<Vec<_>>();

    let ref_path = &fqn_to_path(ref_fqn);
    let call_constructor = WasmConstructor(constructors, ref_path);

    quote! {
        #[no_mangle]
        fn call() {
            let (contract_package_hash, _) = casper_backend::backend::casper_contract::contract_api::storage::create_contract_package_at_hash();
            casper_backend::backend::casper_contract::contract_api::runtime::put_key(#package_hash, contract_package_hash.into());

            #entrypoints

            casper_backend::backend::casper_contract::contract_api::storage::add_contract_version(
                contract_package_hash,
                entry_points,
                casper_backend::backend::casper_types::contracts::NamedKeys::new()
            );

            #call_constructor
        }
    }
}

fn fqn_to_path(fqn: String) -> Path {
    let paths = fqn.split("::").collect::<Vec<_>>();

    let segments = Punctuated::<PathSegment, Token![::]>::from_iter(
        paths
            .iter()
            .map(|ident| PathSegment::from(format_ident!("{}", ident))),
    );

    syn::Path {
        leading_colon: None,
        segments,
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
    use odra::contract_def::{Argument, ContractDef, Entrypoint, EntrypointType};
    use odra::types::CLType;
    use quote::{quote, ToTokens};

    use super::constructor::WasmConstructor;
    use super::entrypoints_def::ContractEntrypoints;
    use super::wasm_entrypoint::WasmEntrypoint;
    use super::{assert_eq_tokens, gen_contract};

    #[test]
    fn test_contract_codegen() {
        let constructor = Entrypoint {
            ident: String::from("construct_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: CLType::I32,
            }],
            ret: CLType::Unit,
            ty: EntrypointType::Constructor,
        };
        let entrypoint = Entrypoint {
            ident: String::from("call_me"),
            args: vec![],
            ret: CLType::Bool,
            ty: EntrypointType::Public,
        };

        let path: syn::Path = syn::parse_str("my_contract::MyContract").unwrap();
        let ref_path: syn::Path = syn::parse_str("my_contract::MyContractRef").unwrap();

        let fqn = path.to_token_stream().to_string().replace(' ', "");

        let contract_def = ContractDef {
            ident: String::from("MyContract"),
            entrypoints: vec![constructor.clone(), entrypoint.clone()],
        };

        let result = gen_contract(contract_def, fqn);

        let expected_constructor_no_mangle = WasmEntrypoint(&constructor, &path);
        let expected_entrypoint_no_mangle = WasmEntrypoint(&entrypoint, &path);
        let entrypoints = vec![constructor.clone(), entrypoint.clone()];
        let expected_entrypoints = ContractEntrypoints(&entrypoints);
        let expected_constructor_if = WasmConstructor(vec![&constructor], &ref_path);

        assert_eq_tokens(
            result,
            quote! {
                #![no_main]

                use odra::Instance;
                use odra_casper_backend as casper_backend;
                #[no_mangle]
                fn call() {
                    let (contract_package_hash , _) = casper_backend::backend::casper_contract::contract_api::storage::create_contract_package_at_hash();
                    casper_backend::backend::casper_contract::contract_api::runtime::put_key(
                        "my_contract_package_hash",
                        contract_package_hash.into()
                    );

                    #expected_entrypoints

                    casper_backend::backend::casper_contract::contract_api::storage::add_contract_version(
                        contract_package_hash,
                        entry_points,
                        casper_backend::backend::casper_types::contracts::NamedKeys::new()
                    );

                    #expected_constructor_if
                }

                #expected_constructor_no_mangle

                #expected_entrypoint_no_mangle
            },
        );
    }
}
