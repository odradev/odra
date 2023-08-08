use std::{fs, path::PathBuf, str::from_utf8_unchecked, time::Duration};

use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b
};
use casper_types::{
    bytesrepr::FromBytes, runtime_args, ContractHash, ContractPackageHash, Key as CasperKey,
    PublicKey, RuntimeArgs
};

use odra_casper_shared::key_maker::KeyMaker;
use odra_casper_types::{Address, Balance, Bytes, CallArgs, OdraType};
use serde::de::DeserializeOwned;
use serde_json::{from_value, json, Value};

use crate::{
    casper_node_port::{
        rpcs::{
            DictionaryIdentifier, GetDeployParams, GetDeployResult, GetDictionaryItemParams,
            GetDictionaryItemResult, GetStateRootHashResult, GlobalStateIdentifier,
            PutDeployResult, QueryGlobalStateParams, QueryGlobalStateResult
        },
        Deploy, DeployHash
    },
    log
};
use crate::casper_node_port::executable_deploy_item::ExecutableDeployItem;
use crate::casper_node_port::hashing::Digest;
use crate::casper_types_port::timestamp::{TimeDiff, Timestamp};

pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";

fn _get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

/// Client for interacting with Casper node.
pub struct CasperClient {
    node_address: String,
    chain_name: String,
    public_key: Option<PublicKey>,
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(node_address: String, chain_name: String) -> Self {
        CasperClient {
            node_address,
            chain_name,
            public_key: None,
        }
    }

    /// Node address.
    pub fn node_address_rpc(&self) -> String {
        format!("{}/rpc", self.node_address)
    }

    /// Chain name.
    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    /// Public key of the client account.
    pub fn public_key(&self) -> PublicKey {
        // TODO: handle the unwrap
        self.public_key.clone().unwrap()
    }

    /// Address of the client account.
    pub fn caller(&self) -> Address {
        Address::from(self.public_key())
    }

    /// Query the node for the current state root hash.
    pub async fn get_state_root_hash(&self) -> Digest {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "chain_get_state_root_hash",
                "id": 1,
            }
        );
        let result: GetStateRootHashResult = self.post_request(request).await.unwrap();
        result.state_root_hash.unwrap()
    }

    /// Query the node for the deploy state.
    pub async fn get_deploy(&self, deploy_hash: DeployHash) -> GetDeployResult {
        let params = GetDeployParams {
            deploy_hash,
            finalized_approvals: false
        };

        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "info_get_deploy",
                "params": params,
                "id": 1,
            }
        );
        self.post_request(request).await.unwrap()
    }

    /// Query the contract for the variable.
    pub async fn get_variable_value<T: OdraType>(&self, address: Address, key: &[u8]) -> Option<T> {
        let key = LivenetKeyMaker::to_variable_key(key);
        // SAFETY: we know the key maker creates a string of valid UTF-8 characters.
        let key = unsafe { from_utf8_unchecked(&key) };
        self.query_dictionary(address, key).await
    }

    /// Query the contract for the dictionary value.
    pub async fn get_dict_value<K: OdraType, V: OdraType>(
        &self,
        address: Address,
        seed: &[u8],
        key: &K
    ) -> Option<V> {
        let key = LivenetKeyMaker::to_dictionary_key(seed, key).unwrap();
        // SAFETY: we know the key maker creates a string of valid UTF-8 characters.
        let key = unsafe { from_utf8_unchecked(&key) };
        self.query_dictionary(address, key).await
    }

    /// Discover the contract address by name.
    pub async fn get_contract_address(&self, key_name: &str) -> Address {
        let key_name = format!("{}_package_hash", key_name);
        let account_hash = self.public_key().to_account_hash();
        let result = self.query_global_state(&CasperKey::Account(account_hash)).await;
        let result_as_json = serde_json::to_value(result.stored_value).unwrap();

        let named_keys = result_as_json["Account"]["named_keys"].as_array().unwrap();
        for named_key in named_keys {
            if named_key["name"].as_str().unwrap() == key_name {
                let key = named_key["key"]
                    .as_str()
                    .unwrap()
                    .replace("hash-", "contract-package-wasm");
                let contract_hash = ContractPackageHash::from_formatted_str(&key).unwrap();
                return Address::try_from(contract_hash).unwrap();
            }
        }
        log::error(format!(
            "Contract {:?} not found in {:#?}",
            key_name, result_as_json
        ));
        panic!("get_contract_address failed");
    }

    /// Find the contract hash by the contract package hash.
    pub async fn query_global_state_for_contract_hash(&self, address: Address) -> ContractHash {
        let key = CasperKey::Hash(address.as_contract_package_hash().unwrap().value());
        let result = self.query_global_state(&key).await;
        let result_as_json = serde_json::to_value(result).unwrap();
        let contract_hash: &str = result_as_json["stored_value"]["ContractPackage"]["versions"][0]
            ["contract_hash"]
            .as_str()
            .unwrap();
        ContractHash::from_formatted_str(contract_hash).unwrap()
    }

    /// Deploy the contract.
    pub async fn deploy_wasm(&self, wasm_file_name: &str, args: CallArgs, gas: Balance) -> Address {
        log::info(format!("Deploying \"{}\".", wasm_file_name));
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

        let response: PutDeployResult = self.post_request(request).await.unwrap();
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash).await;

        let address = self.get_contract_address(wasm_file_name.strip_suffix(".wasm").unwrap()).await;
        log::info(format!("Contract {:?} deployed.", &address.to_string()));
        address
    }

    /// Deploy the entrypoint call.
    pub async fn deploy_entrypoint_call(
        &self,
        addr: Address,
        entrypoint: &str,
        args: &CallArgs,
        _amount: Option<Balance>,
        gas: Balance
    ) {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\".",
            addr.to_string(),
            entrypoint
        ));
        let session = ExecutableDeployItem::StoredVersionedContractByHash {
            hash: *addr.as_contract_package_hash().unwrap(),
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
        let response: PutDeployResult = self.post_request(request).await.unwrap();
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash).await;
    }

    async fn query_global_state(&self, key: &CasperKey) -> QueryGlobalStateResult {
        let state_root_hash = self.get_state_root_hash().await;
        let params = QueryGlobalStateParams {
            state_identifier: GlobalStateIdentifier::StateRootHash(state_root_hash),
            key: key.to_formatted_string(),
            path: Vec::new()
        };
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "query_global_state",
                "params": params,
                "id": 1,
            }
        );
        self.post_request(request).await.unwrap()
    }

    async fn query_dictionary<T: OdraType>(&self, address: Address, key: &str) -> Option<T> {
        let state_root_hash = self.get_state_root_hash().await;
        let contract_hash = self.query_global_state_for_contract_hash(address).await;
        let contract_hash = contract_hash
            .to_formatted_string()
            .replace("contract-", "hash-");
        let params = GetDictionaryItemParams {
            state_root_hash,
            dictionary_identifier: DictionaryIdentifier::ContractNamedKey {
                key: contract_hash,
                dictionary_name: String::from("state"),
                dictionary_item_key: String::from(key)
            }
        };

        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "state_get_dictionary_item",
                "params": params,
                "id": 1,
            }
        );

        let result: Option<GetDictionaryItemResult> = self.post_request(request).await.unwrap();
        result.map(|result| {
            let result_as_json = serde_json::to_value(result).unwrap();
            let result = result_as_json["stored_value"]["CLValue"]["bytes"]
                .as_str()
                .unwrap();
            let bytes = hex::decode(result).unwrap();
            let (value, _) = FromBytes::from_bytes(&bytes).unwrap();
            value
        })
    }

    async fn wait_for_deploy_hash(&self, deploy_hash: DeployHash) {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let time_diff = Duration::from_secs(15);
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &time_diff, &deploy_hash_str
            ));
            std::thread::sleep(time_diff);
            let result: GetDeployResult = self.get_deploy(deploy_hash).await;
            if !result.execution_results.is_empty() {
                final_result = result;
                break;
            }
        }

        match &final_result.execution_results[0].result {
            casper_types::ExecutionResult::Failure {
                effect: _,
                transfers: _,
                cost: _,
                error_message
            } => {
                log::error(format!(
                    "Deploy {:?} failed with error: {:?}.",
                    deploy_hash_str, error_message
                ));
                panic!("Deploy failed");
            }
            casper_types::ExecutionResult::Success {
                effect: _,
                transfers: _,
                cost: _
            } => {
                log::info(format!(
                    "Deploy {:?} successfully executed.",
                    deploy_hash_str
                ));
            }
        }
    }

    pub fn new_deploy(&self, session: ExecutableDeployItem, gas: Balance) -> Deploy {
        // TODO: use real timestamp from the host
        let timestamp = Timestamp::zero();
        let ttl = TimeDiff::from_seconds(1000);
        let gas_price = 1;
        let dependencies = vec![];
        let chain_name = String::from(self.chain_name());
        let payment = ExecutableDeployItem::ModuleBytes {
            module_bytes: Default::default(),
            args: runtime_args! {
                "amount" => gas
            }
        };

        Deploy::new_unsigned(
            timestamp,
            ttl,
            gas_price,
            dependencies,
            chain_name,
            payment,
            session,
            self.public_key()
        )
    }

    async fn post_request<T: DeserializeOwned>(&self, request_value: Value) -> Option<T> {
        web_sys::console::log_1(&request_value.to_string().as_str().into());
        let resp = gloo_net::http::Request::post(&self.node_address_rpc())
            .json(&request_value)
            .unwrap()
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let response_value: Value = serde_json::from_str(&resp).unwrap();
        from_value(response_value["result"].clone()).ok()
        // web_sys::console::log_1(&e.to_string().as_str().into());
    }

}

/// Search for the wasm file in the current directory and in the parent directory.
fn find_wasm_file_path(wasm_file_name: &str) -> PathBuf {
    let mut path = PathBuf::from("wasm").join(wasm_file_name);
    let mut checked_paths = vec![];
    for _ in 0..2 {
        if path.exists() && path.is_file() {
            log::info(format!("Found wasm under {:?}.", path));
            return path;
        } else {
            checked_paths.push(path.clone());
            path = path.parent().unwrap().to_path_buf();
        }
    }
    log::error(format!("Could not find wasm under {:?}.", checked_paths));
    panic!("Wasm not found");
}

pub struct LivenetKeyMaker;

impl KeyMaker for LivenetKeyMaker {
    fn blake2b(preimage: &[u8]) -> [u8; 32] {
        let mut result = [0; 32];
        let mut hasher = VarBlake2b::new(32).expect("should create hasher");

        hasher.update(preimage);
        hasher.finalize_variable(|slice| {
            result.copy_from_slice(slice);
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use casper_types::{
        bytesrepr::{FromBytes, ToBytes},
        U256
    };
    use odra_casper_types::Address;

    use crate::casper_node_port::DeployHash;
    use crate::casper_node_port::hashing::Digest;

    use super::CasperClient;

    const CONTRACT_PACKAGE_HASH: &str =
        "hash-40dd2fef4e994d2b0d3d415ce515446d7a1e389d2e6fc7c51319a70acf6f42d0";
    const ACCOUNT_HASH: &str =
        "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    const NODE_ADDRESS: &str = "http://3.140.179.157:7777";
    const CHAIN_NAME: &str = "integration-test";

    fn client() -> CasperClient {
        CasperClient::new(NODE_ADDRESS.to_string(), CHAIN_NAME.to_string())
    }

    #[tokio::test]
    #[ignore]
    pub async fn client_works() {
        let contract_hash = Address::from_str(CONTRACT_PACKAGE_HASH).unwrap();
        let result: Option<String> =
            client().get_variable_value(contract_hash, b"name_contract").await;
        assert_eq!(result.unwrap().as_str(), "Plascoin");

        let account = Address::from_str(ACCOUNT_HASH).unwrap();
        let balance: Option<U256> =
            client().get_dict_value(contract_hash, b"balances_contract", &account).await;
        assert!(balance.is_some());
    }

    #[tokio::test]
    #[ignore]
    pub async fn state_root_hash() {
        client().get_state_root_hash().await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn get_deploy() {
        let hash = DeployHash::new(
            Digest::from_hex("98de69b3515fbefcd416e09b57642f721db354509c6d298f5f7cfa8b42714dba")
                .unwrap()
        );
        let result = client().get_deploy(hash).await;
        assert_eq!(result.deploy.hash(), &hash);
        client().wait_for_deploy_hash(hash).await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn query_global_state_for_contract() {
        let addr = Address::from_str(CONTRACT_PACKAGE_HASH).unwrap();
        let _result: Option<String> = client().query_dictionary(addr, "name_contract").await;
    }

    #[tokio::test]
    #[ignore]
    pub async fn discover_contract_address() {
        let address = client().get_contract_address("erc20").await;
        let contract_hash = Address::from_str(CONTRACT_PACKAGE_HASH).unwrap();
        assert_eq!(address, contract_hash);
    }

    #[test]
    #[ignore]
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
