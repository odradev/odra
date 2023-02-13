//! Set of functions to generate Casper contract.

use odra_types::contract_def::{ContractDef, Entrypoint, EntrypointType};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod action;
mod entry_point;
mod utils;

const UNIT: &str = "()";

/// Given the ContractDef from Odra, generate Cosmos contract.
pub fn gen_contract(contract_def: ContractDef, fqn: String) -> TokenStream2 {
    let init_filter = |ep: &&Entrypoint| ep.ty == EntrypointType::Constructor;
    let exec_filter = |ep: &&Entrypoint| ep.ty != EntrypointType::Constructor && &ep.ret == UNIT;
    let query_filter = |ep: &&Entrypoint| ep.ty != EntrypointType::Constructor && &ep.ret != UNIT;

    let contract_path = utils::fqn_to_path(&fqn);
    let action_ident = action::ident();
    let action_ty_ident = action::action_ty_ident();

    let init_variant_matching = entry_point::build_variant_matching(
        &contract_def,
        &contract_path,
        init_filter,
        entry_point::to_variant_branch,
        false
    );
    let exec_variant_matching = entry_point::build_variant_matching(
        &contract_def,
        &contract_path,
        exec_filter,
        entry_point::to_variant_branch,
        true
    );
    let query_variant_matching = entry_point::build_variant_matching(
        &contract_def,
        &contract_path,
        query_filter,
        entry_point::query::to_variant_branch,
        false
    );

    let parse_message = entry_point::parse_message();
    let parse_query_message = entry_point::query::parse_message();
    let action_deser = action::deserialization_code();
    let action_struct = action::struct_code();

    quote! {
        #![no_main]
        use odra::UnwrapOrRevert;
        use odra::{types::*, Instance};

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
            Ok(odra::contract_env::get_response())
        }

        fn exe_fn(input: &[u8]) -> Result<odra::cosmos::cosmwasm_std::Response, String> {
            let #action_ident: #action_ty_ident = #parse_message
            match action.name.as_str() {
                #exec_variant_matching
                _ => return Err(String::from("Unknown action"))
            }
            Ok(odra::contract_env::get_response())
        }

        fn query_fn(input: &[u8]) -> odra::cosmos::cosmwasm_std::StdResult<odra::cosmos::cosmwasm_std::Binary> {
            let #action_ident: #action_ty_ident = #parse_query_message

            match action.name.as_str() {
                #query_variant_matching
                _ => return Err(odra::cosmos::cosmwasm_std::StdError::NotFound { kind: String::from("Unknown command")})
            }
        }

        #action_struct
        #action_deser
    }
}
