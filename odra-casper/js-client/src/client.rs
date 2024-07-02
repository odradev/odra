use crate::casper_wallet::CasperWalletProvider;
use odra_casper_rpc_client::casper_client::configuration::CasperClientConfiguration;
use odra_casper_rpc_client::casper_client::CasperClient;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
struct JsClient {
    casper_client: CasperClient
}

impl JsClient {
    pub async fn new(node_address: String, chain_name: String) -> Self {
        let cwp = CasperWalletProvider();
        cwp.requestConnection().await;
        let public_key = cwp.getActivePublicKey().await;
        let client_configuration = CasperClientConfiguration {
            node_address,
            rpc_id: "1.0".to_string(),
            chain_name,
            secret_keys: vec![],
            secret_key_paths: vec![],
            cspr_cloud_auth_token: None
        };
        let client = CasperClient::new(client_configuration);
        Self {
            casper_client: client
        }
    }

    pub async fn get_state_root_hash(&self) -> String {
        self.casper_client.get_state_root_hash().await.to_string()
    }
}
