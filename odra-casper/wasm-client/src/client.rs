use wasm_bindgen::prelude::wasm_bindgen;
use odra_casper_livenet::casper_client::CasperClient;
use odra_casper_livenet::client_env::ClientEnv;

#[wasm_bindgen]
pub fn configure_env(node_address: String, chain_name: String) {
    let client = CasperClient::new(node_address, chain_name);
    ClientEnv::instance_mut().setup_casper_client(client);
}

#[wasm_bindgen]
pub async fn get_state_root_hash() -> String {
    let client = ClientEnv::instance();

    return match client.casper_client() {
        None => {
            "Nie umiem skonstruować casper clienta".to_string()
        }
        Some(client) => {
            client.get_state_root_hash().await.to_string()
        }
    }

    //
    // let digest = client.get_state_root_hash().await;
    // // "dupa".to_string()
    // digest.to_string()
}