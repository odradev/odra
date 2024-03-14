use curve25519_parser::parse_openssl_25519_privkey;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::{CasperClientConfiguration, SecretKey};
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, client!");
}

#[wasm_bindgen]
pub async fn deploy_contract() -> String {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let key_str = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIHRZr1HEgKVbgchuatwA7dCWDWB7QZe+bpDb5dguIyLE
-----END PRIVATE KEY-----";
    let static_key = parse_openssl_25519_privkey(key_str.as_bytes()).unwrap();
    let secret_key = SecretKey::ed25519_from_bytes(static_key.as_bytes()).unwrap();

    let config = CasperClientConfiguration {
        node_address: "http://localhost:3000/http://localhost:11101".to_string(),
        chain_name: "casper-net-1".to_string(),
        secret_keys: vec![secret_key]
    };

    let casper_client = CasperClient::new(config);
    let blocktime = casper_client.get_block_time().await;
    // let blocktime = 123u64;
    blocktime.to_string()
}
