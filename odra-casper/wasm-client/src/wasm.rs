use super::utils::ToSnakeCase;
use crate::casper_wallet::CasperWalletProvider;
use crate::schemas::{assert_contract_exists_in_schema, load_schemas};
use gloo_utils::format::JsValueSerdeExt;
use odra_casper_livenet::casper_node_port::executable_deploy_item::ExecutableDeployItem;
use odra_casper_livenet::casper_types_port::timestamp::Timestamp;
use odra_casper_livenet::client_env::{build_args, ClientEnv};
use odra_casper_types::{Bytes, CallArgs};
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
struct WrappedDeploy {
    deploy: Deploy
}

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
        return Ok(contract_bin);
    }

    Err(format!("Couldn't find {filename} in contract binaries."))
}

pub async fn deploy_wasm(
    contract_name: &str,
    contract_schemas: &str,
    contract_bins: &[u8]
) -> Result<JsValue, String> {
    let schemas = load_schemas(contract_schemas)?;
    assert_contract_exists_in_schema(contract_name, schemas)?;
    let wasm_bytes = load_wasm_bytes(contract_name, contract_bins)?;
    let mut args = CallArgs::new();
    build_args(&mut args, contract_name, None);
    let session_bytes = ExecutableDeployItem::ModuleBytes {
        module_bytes: Bytes::from(wasm_bytes),
        args: args.as_casper_runtime_args().clone()
    };
    let cwp = CasperWalletProvider();
    cwp.requestConnection().await;
    let Some(public_key) = cwp.getActivePublicKey().await.as_string() else { return Err("Couldn't get public key".to_string()) };
    log_1(&JsValue::from(&public_key));

    let unsigned_deploy = ClientEnv::instance().casper_client().unwrap().new_deploy(
        session_bytes,
        100.into(),
        Timestamp::from(js_sys::Date::now() as u64)
    );

    let wrapped_deploy = WrappedDeploy {
        deploy: unsigned_deploy
    };

    let unsigned_deploy_json = JsValue::from_serde(&wrapped_deploy).unwrap_or(JsValue::UNDEFINED);
    web_sys::console::log_1(&unsigned_deploy_json);
    let well_formatted_deploy = deployFromJson(unsigned_deploy_json);
    web_sys::console::log_1(&well_formatted_deploy);
    let stringified_deploy = stringify(&well_formatted_deploy).unwrap().as_string().unwrap();
    let signed_deploy = cwp.sign(stringified_deploy, public_key).await;
    // Ok(signed_deploy)
    Err("Not implemented".to_string())
}

fn load_bins(contract_bins: &[u8]) -> HashMap<String, Vec<u8>> {
    let bins: HashMap<String, Vec<u8>> = match bincode::deserialize(contract_bins) {
        Ok(bins) => bins,
        Err(_) => panic!("Error parsing contract bins")
    };
    bins
}

use chrono::{DateTime, NaiveDateTime, Utc};
use js_sys::JSON::stringify;
use regex::Regex;
use web_sys::console::log_1;
use odra_casper_livenet::casper_node_port::Deploy;
use crate::deploy_util::{deployFromJson};

fn fix_timestamp_and_ttl(json: String) -> String {
    fix_timestamp(fix_ttl(json))
}

fn fix_timestamp(json: String) -> String {
    let timestamp_regex = Regex::new(r#""timestamp":(\d+)"#).unwrap();

    timestamp_regex.replace_all(&json, |caps: &regex::Captures| {
        let timestamp = caps.get(1).unwrap().as_str().parse::<i64>().unwrap() / 1000;
        let timestamp = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_millis(timestamp).unwrap(), Utc);

        format!(r#""timestamp": "{}""#, timestamp.to_rfc3339())
    }).to_string()
}

fn fix_ttl(json: String) -> String {
    let timestamp_regex = Regex::new(r#""ttl":(\d+)"#).unwrap();

    timestamp_regex.replace_all(&json, |caps: &regex::Captures| {
        let ttl = caps.get(2).unwrap().as_str().parse::<i64>().unwrap() / 60000;
        let ttl = format!("{}m", ttl);

        format!(r#""ttl": "{}""#, ttl)
    }).to_string()
}