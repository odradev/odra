use std::collections::HashMap;
use odra_casper_livenet::client_env::unsigned_deploy_json;
use crate::imports::sign_json;
use crate::schemas::{assert_contract_exists_in_schema, load_schemas};
use super::utils::ToSnakeCase;

pub fn load_bin(contract_name: &str, contract_bins: &[u8]) -> Result<Vec<u8>, String> {
    let bins = load_bins(contract_bins);
    let filename = format!("{}.wasm", contract_name.to_snake_case());

    let mut contract_bin = Vec::new();
    bins.iter().for_each(|(name, bin)| {
        if name == &filename {
            contract_bin = bin.clone()
        }
    });

    if !contract_bin.is_empty() {
        return Ok(contract_bin)
    }

    Err(format!("Couldn't find {filename} in contract binaries."))
}

pub fn deploy_wasm(contract_name: &str, contract_schemas: &str, contract_bins: &[u8]) -> Result<(), String>{
    let schemas = load_schemas(contract_schemas)?;
    assert_contract_exists_in_schema(contract_name, schemas)?;
    let bin = load_bin(contract_name, contract_bins)?;
    let unsigned_deploy = unsigned_deploy_json(bin);
    let signed_deploy = sign_json(unsigned_deploy);
    Err(signed_deploy)
}

fn load_bins(contract_bins: &[u8]) -> HashMap<String, Vec<u8>> {
    let bins: HashMap<String, Vec<u8>> = match bincode::deserialize(contract_bins) {
        Ok(bins) => bins,
        Err(_) => panic!("Error parsing contract bins"),
    };
    bins
}