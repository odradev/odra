use std::collections::HashMap;
use odra_casper_livenet::casper_node_port::executable_deploy_item::ExecutableDeployItem;
use odra_casper_livenet::client_env::{build_args, ClientEnv};
use odra_casper_types::{Bytes, CallArgs};
use crate::casper_wallet::CasperWalletProvider;
use crate::schemas::{assert_contract_exists_in_schema, load_schemas};
use super::utils::ToSnakeCase;

pub fn load_wasm_bytes(contract_name: &str, contract_bins: &[u8]) -> Result<Vec<u8>, String> {
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
    let wasm_bytes = load_wasm_bytes(contract_name, contract_bins)?;
    let mut args = CallArgs::new();
    build_args(&mut args, contract_name, None);
    let session_bytes = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::from(wasm_bytes),
        args: args.as_casper_runtime_args().clone(),
    };
    let unsigned_deploy = ClientEnv::instance().casper_client().unwrap().new_deploy(session_bytes, 100.into());
    let cwp = CasperWalletProvider();
    let signed_deploy = cwp.signDeploy(serde_json::to_string(&unsigned_deploy).unwrap(), cwp.getActivePublicKey());
    Err(signed_deploy)
}

fn load_bins(contract_bins: &[u8]) -> HashMap<String, Vec<u8>> {
    let bins: HashMap<String, Vec<u8>> = match bincode::deserialize(contract_bins) {
        Ok(bins) => bins,
        Err(_) => panic!("Error parsing contract bins"),
    };
    bins
}
