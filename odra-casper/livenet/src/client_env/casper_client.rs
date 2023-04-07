use casper_client::{self, GlobalStateStrParams};
use casper_types::bytesrepr::FromBytes;
use odra_casper_types::{Address, OdraType};
use serde_json::Value;

pub struct CasperClient {
    node_address: String
}

impl CasperClient {
    pub fn intergration_testnet() -> Self {
        CasperClient {
            node_address: String::from("http://3.140.179.157:7777")
        }
    }

    pub fn get_var<T: OdraType>(&self, address: Address, key: &str) -> Option<T> {
        let state_root_hash = self.get_state_root_hash();
        let contract_hash = address.to_string();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            // Turn contract_package_hash into a contract_hash so it can be queried.
            // let response = casper_client::get_item("",
            //     &self.node_address, 0, &state_root_hash, &contract_hash, "").await;
            let state_root = GlobalStateStrParams {
                is_block_hash: false,
                hash_value: &state_root_hash
            };
            let response = casper_client::query_global_state(
                "",
                &self.node_address,
                0,
                state_root,
                &contract_hash,
                ""
            )
            .await;
            let response = response.unwrap();
            let response: &Value = response.get_result().unwrap();
            let contract_hash: &str = response["stored_value"]["ContractPackage"]["versions"][0]
                ["contract_hash"]
                .as_str()
                .unwrap();
            let contract_hash = contract_hash.replace("contract-", "hash-");

            // Query contract.
            let state_root = GlobalStateStrParams {
                is_block_hash: false,
                hash_value: &state_root_hash
            };
            let response = casper_client::query_global_state(
                "",
                &self.node_address,
                0,
                state_root,
                &contract_hash,
                key
            )
            .await;
            let response = response.unwrap();
            let response: &Value = response.get_result().unwrap();
            let bytes: &str = response["stored_value"]["CLValue"]["bytes"]
                .as_str()
                .unwrap();
            let bytes = hex::decode(bytes).unwrap();
            let (value, _) = FromBytes::from_bytes(&bytes).unwrap();
            Some(value)
        })
    }

    pub fn get_state_root_hash(&self) -> String {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async {
            casper_client::get_state_root_hash("", &self.node_address, 0, "").await
        });
        let result = result.unwrap();
        let result: &Value = result.get_result().unwrap();
        let hash = result["state_root_hash"].as_str().unwrap();
        String::from(hash)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use casper_types::bytesrepr::{FromBytes, ToBytes};
    use odra_casper_types::Address;

    use crate::client_env::casper_client::CasperClient;

    const HASH: &str = "hash-57cbf48566ee3f59b2ae8ef45d28a49a462899f9765c2c15921b4ac5197a2f4d";

    #[test]
    pub fn client_works() {
        let contract_hash = Address::from_str(HASH).unwrap();
        let result: Option<String> =
            CasperClient::intergration_testnet().get_var(contract_hash, "name");
        assert_eq!(result.unwrap().as_str(), "DragonsNFT_2");
    }

    #[test]
    pub fn state_root_hash() {
        CasperClient::intergration_testnet().get_state_root_hash();
    }

    #[test]
    pub fn parsing() {
        let name = String::from("DragonsNFT_2");
        let bytes = ToBytes::to_bytes(&name).unwrap();
        assert_eq!(
            hex::encode(bytes.clone()),
            "0c000000447261676f6e734e46545f32"
        );
        let (name2, _): (String, _) = FromBytes::from_bytes(&bytes).unwrap();
        assert_eq!(name, name2);
    }
}
