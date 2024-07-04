use odra_casper_js_client::client::OdraClient;
use wasm_bindgen::prelude::*;
const CONTRACT_SCHEMAS: &str = include_str!("../contracts.json");

#[wasm_bindgen]
pub fn schemas() -> String {
    CONTRACT_SCHEMAS.to_string()
}

#[wasm_bindgen]
pub async fn test_client() -> String {
    console_error_panic_hook::set_once();
    const CONTRACT_PACKAGE_HASH: &str =
        "hash-40dd2fef4e994d2b0d3d415ce515446d7a1e389d2e6fc7c51319a70acf6f42d0";
    const ACCOUNT_HASH: &str =
        "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    const NODE_ADDRESS: &str = "http://localhost:3000/http://localhost:11101";
    const CHAIN_NAME: &str = "casper-net-1";
    let client = OdraClient::new(NODE_ADDRESS.to_string(), CHAIN_NAME.to_string()).await;
    let state_root_hash = client.get_state_root_hash().await;
    state_root_hash
}
