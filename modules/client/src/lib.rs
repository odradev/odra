use casper_types::{DeployHash, Key, U256};
use wasm_bindgen::prelude::*;
use odra_casper_livenet::casper_client::CasperClient;
use odra_casper_types::{Address};
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CONTRACT_SCHEMAS: &str = include_str!("../contracts.json");

#[wasm_bindgen]
pub fn erc20_deployer(name: String, symbol: String, decimals: u8, initial_supply: Option<Vec<u8>>) -> Vec<u8> {
    DeployHash::new([0u8; 32]).as_bytes().to_vec()
}

#[wasm_bindgen]
pub fn setup() {

}

#[wasm_bindgen]
pub fn deploy_contract() {

}

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
    const NODE_ADDRESS: &str = "http://localhost:30800/http://3.140.179.157:7777";
    const CHAIN_NAME: &str = "integration-test";
    let client = CasperClient::new(NODE_ADDRESS.to_string(), CHAIN_NAME.to_string(), None);

    let key = Key::from_formatted_str(CONTRACT_PACKAGE_HASH).unwrap();
    let contract_hash = Address::try_from(key).unwrap();
        let result: Option<String> =
            client.get_variable_value(contract_hash, "name_contract").await;

    result.unwrap()
        // assert_eq!(result.unwrap().as_str(), "Plascoin");
        //
        // let account = Address::from_str(ACCOUNT_HASH).unwrap();
        // let balance: Option<U256> =
        //     client.get_dict_value(contract_hash, "balances_contract", &account);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
