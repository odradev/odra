use curve25519_parser::parse_openssl_25519_privkey;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::{CasperClientConfiguration, SecretKey};
use std::panic;
use wasm_bindgen::prelude::*;
use odra::Address;
use odra::casper_types::bytesrepr::{Bytes, ToBytes};
use odra::casper_types::runtime_args;
use crate::runtime_args::RuntimeArgs;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, client!");
}

#[wasm_bindgen]
pub async fn deploy_contract(timestamp: u64) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let wasm_bytes = include_bytes!("../../../examples/wasm/Erc20.wasm");
    // prepare build.rs to load key from file
    let key_bytes = [
        116,
    89,
    175,
    81,
    196,
    128,
    165,
    91,
    129,
    200,
    110,
    106,
    220,
    0,
    237,
    208,
    150,
    13,
    96,
    123,
    65,
    151,
    190,
    110,
    144,
    219,
    229,
    216,
    46,
    35,
    34,
    196,
];

    let secret_key = SecretKey::ed25519_from_bytes(key_bytes).unwrap();
    let config = CasperClientConfiguration {
        node_address: "http://localhost:3000/http://localhost:11101".to_string(),
        chain_name: "casper-net-1".to_string(),
        secret_keys: vec![secret_key]
    };

    let mut casper_client = CasperClient::new(config);
    casper_client.set_gas(30_000_000_000u64);
    let address = casper_client.deploy_wasm_bytes("Erc20", Bytes::from(wasm_bytes.as_slice()), runtime_args! {
        "name" => "Plascoin",
        "symbol" => "PLS",
        "decimals" => 18,
        "initial_supply" => 1000000u64

    }, timestamp).await;

    JsValue::from(address.to_string())
}
