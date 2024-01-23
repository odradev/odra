use std::{fs, path::PathBuf, str::from_utf8_unchecked, time::Duration};

use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_hashing::Digest;
use jsonrpc_lite::JsonRpc;
use odra_core::casper_types::{URef, URefAddr};
use odra_core::CLType::U32;
use odra_core::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        runtime_args, ContractHash, ContractPackageHash, ExecutionResult, Key as CasperKey,
        PublicKey, RuntimeArgs, SecretKey, TimeDiff, Timestamp, U512
    },
    consts::*,
    Address, CLTyped, CallDef, ExecutionError, OdraError
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::casper_node_port::rpcs::StoredValue::CLValue;
use crate::casper_node_port::{
    rpcs::{
        DictionaryIdentifier, GetDeployParams, GetDeployResult, GetDictionaryItemParams,
        GetDictionaryItemResult, GetStateRootHashResult, GlobalStateIdentifier, PutDeployResult,
        QueryGlobalStateParams, QueryGlobalStateResult
    },
    Deploy, DeployHash
};

use crate::{casper_node_port, log};

pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";

fn get_env_variable(name: &str) -> String {
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
    secret_key: SecretKey,
    gas: U512
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        CasperClient {
            node_address: get_env_variable(ENV_NODE_ADDRESS),
            chain_name: get_env_variable(ENV_CHAIN_NAME),
            secret_key: SecretKey::from_file(get_env_variable(ENV_SECRET_KEY)).unwrap(),
            gas: U512::zero()
        }
    }

    pub fn get_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.query_state_dictionary(address, unsafe { from_utf8_unchecked(key) })
    }

    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas.into();
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
        PublicKey::from(&self.secret_key)
    }

    /// Address of the client account.
    pub fn caller(&self) -> Address {
        Address::from(self.public_key())
    }

    pub fn get_block_time(&self) -> u64 {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "info_get_status",
                "id": 1,
            }
        );
        let result: serde_json::Value = self.post_request(request).unwrap();
        let result = result["result"]["last_added_block_info"]["timestamp"]
            .as_str()
            .unwrap();
        let timestamp: u64 = result.parse().unwrap();
        timestamp
    }

    /// Query the node for the current state root hash.
    pub fn get_state_root_hash(&self) -> Digest {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "chain_get_state_root_hash",
                "id": 1,
            }
        );
        let result: GetStateRootHashResult = self.post_request(request).unwrap();
        result.state_root_hash.unwrap()
    }

    pub fn query_dict<T: FromBytes + CLTyped>(
        &self,
        contract_address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<T, OdraError> {
        let state_root_hash = self.get_state_root_hash();
        let contract_hash = self.query_global_state_for_contract_hash(contract_address);
        let contract_hash = contract_hash
            .to_formatted_string()
            .replace("contract-", "hash-");
        let params = GetDictionaryItemParams {
            state_root_hash,
            dictionary_identifier: DictionaryIdentifier::ContractNamedKey {
                key: contract_hash,
                dictionary_name,
                dictionary_item_key
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
        let result: GetDictionaryItemResult = self.post_request(request).unwrap();
        match result.stored_value {
            CLValue(value) => {
                let return_value: T = value.into_t().unwrap();
                Ok(return_value)
            }
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    pub fn query_contract_named_key<T: AsRef<str>>(
        &self,
        contract_address: &Address,
        key: T
    ) -> Result<String, OdraError> {
        let d = self.query_global_state_path(contract_address, key.as_ref().to_string());
        let c = d.as_ref().unwrap().stored_value.clone();
        Ok(match d.as_ref().unwrap().stored_value.clone() {
            casper_node_port::rpcs::StoredValue::Contract(contract) => {
                contract.named_keys().find_map(|named_key| {
                    if named_key.name == key.as_ref() {
                        Some(named_key.key.clone())
                    } else {
                        None
                    }
                })
            }
            _ => panic!("Not a contract")
        }
        .unwrap())
    }

    /// Query the node for the deploy state.
    pub fn get_deploy(&self, deploy_hash: DeployHash) -> GetDeployResult {
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
        self.post_request(request).unwrap()
    }

    /// Discover the contract address by name.
    pub fn get_contract_address(&self, key_name: &str) -> Address {
        let key_name = format!("{}_package_hash", key_name);
        let account_hash = self.public_key().to_account_hash();
        let result = self.query_global_state(&CasperKey::Account(account_hash));
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
    pub fn query_global_state_for_contract_hash(&self, address: &Address) -> ContractHash {
        let key = CasperKey::Hash(address.as_contract_package_hash().unwrap().value());
        let result = self.query_global_state(&key);
        let result_as_json = serde_json::to_value(result).unwrap();
        let contract_hash: &str = result_as_json["stored_value"]["ContractPackage"]["versions"][0]
            ["contract_hash"]
            .as_str()
            .unwrap();
        ContractHash::from_formatted_str(contract_hash).unwrap()
    }

    /// Deploy the contract.
    pub fn deploy_wasm(&self, contract_name: &str, args: RuntimeArgs) -> Address {
        log::info(format!("Deploying \"{}\".", contract_name));
        let wasm_path = find_wasm_file_path(contract_name);
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::from(wasm_bytes),
            args
        };
        let deploy = self.new_deploy(session, self.gas);
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

        let response: PutDeployResult = self.post_request(request).unwrap();
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);

        let address = self.get_contract_address(contract_name);
        log::info(format!("Contract {:?} deployed.", &address.to_string()));
        address
    }

    pub fn deploy_entrypoint_call_with_proxy(&self, addr: Address, call_def: CallDef) -> Bytes {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\".",
            addr.to_string(),
            call_def.entry_point
        ));

        let args = runtime_args! {
            CONTRACT_PACKAGE_HASH_ARG => *addr.as_contract_package_hash().unwrap(),
            ENTRY_POINT_ARG => call_def.entry_point,
            ARGS_ARG => Bytes::from(call_def.args.to_bytes().unwrap()),
            ATTACHED_VALUE_ARG => call_def.amount,
            AMOUNT_ARG => call_def.amount,
        };

        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: include_bytes!("../../test-vm/resources/proxy_caller_with_return.wasm")
                .to_vec()
                .into(),
            args
        };

        let deploy = self.new_deploy(session, self.gas);
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
        let response: PutDeployResult = self.post_request(request).unwrap();
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);

        let r = self.query_global_state(&CasperKey::Account(self.public_key().to_account_hash()));
        let result_as_json = serde_json::to_value(r).unwrap();
        let result = result_as_json["stored_value"]["CLValue"]["bytes"]
            .as_str()
            .unwrap();
        let bytes = hex::decode(result).unwrap();
        let (value, _) = FromBytes::from_bytes(&bytes).unwrap();
        value
    }

    /// Deploy the entrypoint call.
    pub fn deploy_entrypoint_call(&self, addr: Address, call_def: CallDef) {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\".",
            addr.to_string(),
            call_def.entry_point
        ));
        let session = ExecutableDeployItem::StoredVersionedContractByHash {
            hash: *addr.as_contract_package_hash().unwrap(),
            version: None,
            entry_point: call_def.entry_point,
            args: call_def.args
        };
        let deploy = self.new_deploy(session, self.gas);
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
        let response: PutDeployResult = self.post_request(request).unwrap();
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);
    }

    pub fn query_global_state_path(
        &self,
        address: &Address,
        path: String
    ) -> Option<QueryGlobalStateResult> {
        let hash = self.query_global_state_for_contract_hash(address);
        let key = CasperKey::Hash(hash.value());
        let state_root_hash = self.get_state_root_hash();
        let params = QueryGlobalStateParams {
            state_identifier: GlobalStateIdentifier::StateRootHash(state_root_hash),
            key: key.to_formatted_string(),
            path: vec![]
        };
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "query_global_state",
                "params": params,
                "id": 1,
            }
        );
        self.post_request(request).unwrap()
    }

    pub fn query_global_state(&self, key: &CasperKey) -> QueryGlobalStateResult {
        let state_root_hash = self.get_state_root_hash();
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
        self.post_request(request).unwrap()
    }

    fn query_state_dictionary(&self, address: &Address, key: &str) -> Option<Bytes> {
        let state_root_hash = self.get_state_root_hash();
        let contract_hash = self.query_global_state_for_contract_hash(address);
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
        let result: Option<GetDictionaryItemResult> = self.post_request(request);
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

    pub fn query_uref<T: CLTyped + FromBytes>(&self, uref: URef) -> Result<T, OdraError> {
        let result = self.query_global_state(&CasperKey::URef(uref));
        match result.stored_value {
            CLValue(value) => {
                let return_value: T = value.into_t().unwrap();
                Ok(return_value)
            }
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    fn wait_for_deploy_hash(&self, deploy_hash: DeployHash) -> ExecutionResult {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let time_diff = Duration::from_secs(15);
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &time_diff, &deploy_hash_str
            ));
            std::thread::sleep(time_diff);
            let result: GetDeployResult = self.get_deploy(deploy_hash);
            if !result.execution_results.is_empty() {
                final_result = result;
                break;
            }
        }

        match &final_result.execution_results[0].result {
            ExecutionResult::Failure {
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
            ExecutionResult::Success {
                effect: _,
                transfers: _,
                cost: _
            } => {
                log::info(format!(
                    "Deploy {:?} successfully executed.",
                    deploy_hash_str
                ));
                final_result.execution_results[0].result.clone()
            }
        }
    }

    fn new_deploy(&self, session: ExecutableDeployItem, gas: U512) -> Deploy {
        let timestamp = Timestamp::now();
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

    fn post_request<T: DeserializeOwned>(&self, request: Value) -> Option<T> {
        // TODO: change result to Result<T, OdraError>
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(self.node_address_rpc())
            .json(&request)
            .send()
            .unwrap();
        let response: JsonRpc = response.json().unwrap();
        response
            .get_result()
            .map(|result| serde_json::from_value(result.clone()).unwrap())
    }
}

impl Default for CasperClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Search for the wasm file in the current directory and in the parent directory.
fn find_wasm_file_path(wasm_file_name: &str) -> PathBuf {
    let mut path = PathBuf::from("wasm")
        .join(wasm_file_name)
        .with_extension("wasm");
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use casper_hashing::Digest;
    use odra_core::{
        casper_types::bytesrepr::{FromBytes, ToBytes},
        Address, U256
    };

    use crate::casper_node_port::DeployHash;

    use super::CasperClient;

    const CONTRACT_PACKAGE_HASH: &str =
        "hash-40dd2fef4e994d2b0d3d415ce515446d7a1e389d2e6fc7c51319a70acf6f42d0";
    const ACCOUNT_HASH: &str =
        "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";

    #[test]
    #[ignore]
    pub fn state_root_hash() {
        CasperClient::new().get_state_root_hash();
    }

    #[test]
    #[ignore]
    pub fn get_deploy() {
        let hash = DeployHash::new(
            Digest::from_hex("98de69b3515fbefcd416e09b57642f721db354509c6d298f5f7cfa8b42714dba")
                .unwrap()
        );
        let result = CasperClient::new().get_deploy(hash);
        assert_eq!(result.deploy.hash(), &hash);
        CasperClient::new().wait_for_deploy_hash(hash);
    }

    #[test]
    #[ignore]
    pub fn query_global_state_for_contract() {
        let addr = Address::from_str(CONTRACT_PACKAGE_HASH).unwrap();
        let _result: Option<String> =
            CasperClient::new().query_state_dictionary(addr, "name_contract");
    }

    #[test]
    #[ignore]
    pub fn discover_contract_address() {
        let address = CasperClient::new().get_contract_address("erc20");
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
