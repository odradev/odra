use std::{time::Duration, path::PathBuf, fs::{File, self}, sync::{Arc, Mutex}};

use casper_client::{self, GlobalStateStrParams};
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_node::{types::{Deploy, Timestamp, TimeDiff, DeployHash, json_compatibility::Account}, crypto::AsymmetricKeyExt, rpcs::{info::GetDeployResult, account::PutDeployResult}};
use casper_types::{bytesrepr::FromBytes, runtime_args, RuntimeArgs, U512, SecretKey, PublicKey, ContractPackageHash};
use jsonrpc_lite::JsonRpc;
use odra_casper_types::{Address, OdraType, Balance, CallArgs, Bytes};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

pub struct CasperClient {
    node_address: String,
    secret_key: SecretKey
}

impl CasperClient {
    pub fn intergration_testnet() -> Self {
        let secret_key_path = "/home/ziel/workspace/dao/testnet-dao/integration-keys/secret_key.pem";
        CasperClient {
            node_address: String::from("http://3.140.179.157:7777"),
            secret_key: SecretKey::from_file(secret_key_path).unwrap()
        }
    }

    pub fn node_address_rpc(&self) -> String {
        format!("{}/rpc", self.node_address)
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.secret_key)
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

    pub fn put_deploy(&self, addr: Address, entrypoint: &str, args: CallArgs, amount: Option<Balance>, gas: Balance) {
        let session = ExecutableDeployItem::StoredVersionedContractByHash { 
            hash: addr.as_contract_package_hash().unwrap().clone(),
            version: None,
            entry_point: String::from(entrypoint),
            args: args.as_casper_runtime_args().clone()
        };
        let deploy = self.new_deploy(session, gas);
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "account_put_deploy",
                "params": {
                    "deploy": deploy
                },
                "id": 1,
            }
        );
        let response: PutDeployResult = self.post_request(request);
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);
    }

    pub fn wait_for_deploy_hash(&self, deploy_hash: DeployHash) {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let time_diff = Duration::from_secs(5);
        let final_result; 
        loop {
            println!("Waiting {:?} for {:?}", &time_diff, &deploy_hash_str);
            std::thread::sleep(time_diff);
            let result = runtime.block_on(async {
                casper_client::get_deploy("", &self.node_address, 0, &deploy_hash_str).await
            });
            let result = result.unwrap();
            let result: &Value = result.get_result().unwrap();
            let result: GetDeployResult = serde_json::from_value(result.clone()).unwrap();
            if result.execution_results.len() > 0 {
                final_result = result;
                break;
            }
        }
        
        match &final_result.execution_results[0].result {
            casper_types::ExecutionResult::Failure { effect: _, transfers: _, cost: _, error_message } => {
                panic!("Deploy {:?} failed with error: {:?}", deploy_hash_str, error_message);
            },
            casper_types::ExecutionResult::Success { effect: _, transfers: _, cost: _ } => {
                println!("Deploy {:?} successfully executed", deploy_hash_str);
            },
        }
    }

    pub fn deploy_wasm(&self, wasm_file_name: &str, args: CallArgs, gas: Balance) -> Address {
        println!("Deploying new wasm file {}", wasm_file_name);
        let wasm_path = find_wasm_file_path(wasm_file_name);
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::from(wasm_bytes),
            args: args.as_casper_runtime_args().clone()
        };
        let deploy = self.new_deploy(session, gas);
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "account_put_deploy",
                "params": {
                    "deploy": deploy
                },
                "id": 1,
            }
        );
        let response: PutDeployResult = self.post_request(request);
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);

        let address = self.get_contract_address(wasm_file_name.strip_suffix(".wasm").unwrap());
        println!("Contract address: {:?}", &address);
        address
    }

    fn get_contract_address(&self, key_name: &str) -> Address {
        let account_hash = self.public_key().to_account_hash();
        let state_root_hash = self.get_state_root_hash();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let state_root = GlobalStateStrParams {
                is_block_hash: false,
                hash_value: &state_root_hash
            };
            let response = casper_client::query_global_state(
                "",
                &self.node_address,
                0,
                state_root,
                &account_hash.to_formatted_string(),
                ""
            )
            .await;
            let key_name = format!("{}_package_hash", key_name);
            let response = response.unwrap();
            let response = response.get_result().unwrap();
            let named_keys = response["stored_value"]["Account"]["named_keys"].as_array().unwrap();
            for named_key in named_keys {
                if named_key["name"].as_str().unwrap() == key_name {
                    let key = named_key["key"].as_str().unwrap().replace("hash-", "contract-package-wasm");
                    let contract_hash = ContractPackageHash::from_formatted_str(&key).unwrap();
                    return Address::try_from(contract_hash).unwrap();
                }
            }
            panic!("Contract package hash not found in global state. \n Named keys: {:#?}", named_keys);
        })
    }

    fn new_deploy(&self, session: ExecutableDeployItem, gas: Balance) -> Deploy {
        let timestamp = Timestamp::now();
        let ttl = TimeDiff::from(10000000);
        let gas_price = 1;
        let dependencies = vec![];
        let chain_name = String::from("integration-test");
        let payment = ExecutableDeployItem::ModuleBytes {
            module_bytes: Default::default(),
            args: runtime_args! {
                "amount" => gas
            }
        };

        Deploy::new(
            timestamp,
            ttl,
            gas_price,
            dependencies,
            chain_name,
            payment,
            session,
            &self.secret_key,
            Some(self.public_key())
        )
    }

    fn post_request<T: DeserializeOwned>(&self, request: Value) -> T {
        let client = reqwest::blocking::Client::new();
        let response = client.post(&self.node_address_rpc()).json(&request).send().unwrap();
        let response: JsonRpc = response.json().unwrap();
        let response = response.get_result().unwrap();
        serde_json::from_value(response.clone()).unwrap()
    }
}

fn find_wasm_file_path(wasm_file_name: &str) -> PathBuf {
    let mut path = PathBuf::from("wasm").join(wasm_file_name);
    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() {
            println!("Found wasm under {:?}", path);
            return path;
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    panic!("Could not find wasm under {:?}", checked_paths);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use casper_types::bytesrepr::{FromBytes, ToBytes};
    use odra_casper_types::Address;

    use super::CasperClient;

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
