use wasm_bindgen::prelude::wasm_bindgen;
use odra_casper_livenet::casper_client::CasperClient;
use odra_casper_livenet::client_env::ClientEnv;
use crate::casper_wallet::CasperWalletProvider;

#[wasm_bindgen]
pub async fn configure_env(node_address: String, chain_name: String) {
    let cwp = CasperWalletProvider();
    cwp.requestConnection().await;
    let public_key = cwp.getActivePublicKey().await;
    let client = CasperClient::new(node_address, chain_name, public_key.as_string().unwrap());
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