//! Client for interacting with Casper node.
use std::collections::BTreeMap;
use std::time::SystemTime;
use std::{fs, path::PathBuf, str::from_utf8_unchecked, time::Duration};

use anyhow::Context;
use casper_execution_engine::core::engine_state::ExecutableDeployItem;
use casper_hashing::Digest;
use itertools::Itertools;
use jsonrpc_lite::JsonRpc;
use odra_core::OdraResult;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::casper_client::configuration::CasperClientConfiguration;
use crate::casper_node_port::query_balance::{
    PurseIdentifier, QueryBalanceParams, QueryBalanceResult, QUERY_BALANCE_METHOD
};
use crate::casper_node_port::rpcs::StoredValue::*;
use crate::casper_node_port::{
    rpcs::{
        DictionaryIdentifier, GetDeployParams, GetDeployResult, GetDictionaryItemParams,
        GetDictionaryItemResult, GetStateRootHashResult, GlobalStateIdentifier, PutDeployResult,
        QueryGlobalStateParams, QueryGlobalStateResult
    },
    Deploy, DeployHash
};
use crate::log;
use anyhow::Result;
use odra_core::casper_types::{sign, URef};
use odra_core::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        runtime_args, CLTyped, ContractHash, ContractPackageHash, ExecutionResult,
        Key as CasperKey, PublicKey, RuntimeArgs, SecretKey, TimeDiff, Timestamp, U512
    },
    consts::*,
    Address, CallDef, ExecutionError, OdraError
};
use tokio::time::sleep;

pub mod configuration;
mod error;

/// Environment variable holding a path to a secret key of a main account.
pub const ENV_SECRET_KEY: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";
/// Environment variable holding an address of the casper node exposing RPC API.
pub const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
/// Environment variable holding a name of the chain.
pub const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
/// Environment variable holding a filename prefix for additional accounts.
pub const ENV_ACCOUNT_PREFIX: &str = "ODRA_CASPER_LIVENET_KEY_";
/// Environment variable holding cspr.cloud auth token.
pub const ENV_CSPR_CLOUD_AUTH_TOKEN: &str = "CSPR_CLOUD_AUTH_TOKEN";
/// Environment variable holding a path to an additional .env file.
pub const ENV_LIVENET_ENV_FILE: &str = "ODRA_CASPER_LIVENET_ENV";

enum ContractId {
    Name(String),
    Address(Address)
}

/// Client for interacting with Casper node.
pub struct CasperClient {
    configuration: CasperClientConfiguration,
    active_account: usize,
    gas: U512,
    contracts: BTreeMap<Address, String>
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(configuration: CasperClientConfiguration) -> Self {
        CasperClient {
            configuration,
            active_account: 0,
            gas: U512::zero(),
            contracts: BTreeMap::new()
        }
    }

    /// Gets a value from the storage
    pub async fn get_value(&self, address: &Address, key: &[u8]) -> Result<Bytes> {
        self.query_state_dictionary(address, unsafe { from_utf8_unchecked(key) })
            .await
    }

    /// Gets a value from a named key
    pub async fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        let uref = self.query_contract_named_key(address, name).await.unwrap();
        self.query_uref_bytes(uref).await.ok()
    }

    /// Gets a value from a named dictionary
    pub async fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        let key = String::from_utf8(key.to_vec()).unwrap();
        self.query_dict_bytes(address, dictionary_name.to_string(), key)
            .await
            .ok()
    }

    /// Sets amount of gas for the next deploy.
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas.into();
    }

    /// Node address.
    pub fn node_address_rpc(&self) -> String {
        format!("{}/rpc", self.configuration.node_address)
    }

    /// Chain name.
    pub fn chain_name(&self) -> &str {
        &self.configuration.chain_name
    }

    /// Public key of the client account.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from(&self.configuration.secret_keys[self.active_account])
    }

    /// Public key of the account address.
    pub fn address_public_key(&self, address: &Address) -> PublicKey {
        PublicKey::from(self.address_secret_key(address))
    }

    /// Secret key of the client account.
    pub fn secret_key(&self) -> &SecretKey {
        &self.configuration.secret_keys[self.active_account]
    }

    /// Signs the message using keys associated with an address.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> OdraResult<Bytes> {
        let secret_key = self.address_secret_key(address);
        let public_key = &PublicKey::from(secret_key);
        let signature = sign(message, secret_key, public_key)
            .to_bytes()
            .map_err(|_| OdraError::ExecutionError(ExecutionError::CouldNotSignMessage))?;

        Ok(Bytes::from(signature))
    }

    /// Address of the client account.
    pub fn caller(&self) -> Address {
        Address::from(self.public_key())
    }

    /// Address of the account loaded to the client.
    pub fn get_account(&self, index: usize) -> Address {
        if index >= self.secret_keys().len() {
            panic!("Key for account with index {} is not loaded", index);
        }
        Address::from(PublicKey::from(&self.secret_keys()[index]))
    }

    /// Sets the caller account.
    pub fn set_caller(&mut self, address: Address) {
        match self
            .secret_keys()
            .iter()
            .find_position(|key| Address::from(PublicKey::from(*key)) == address)
        {
            Some((index, _)) => {
                self.active_account = index;
            }
            None => panic!("Key for address {:?} is not loaded", address)
        }
    }

    /// Returns the balance of the account.
    pub async fn get_balance(&self, address: &Address) -> U512 {
        let query_balance_params = QueryBalanceParams::new(
            Some(GlobalStateIdentifier::StateRootHash(
                self.get_state_root_hash().await
            )),
            PurseIdentifier::MainPurseUnderAccountHash(
                *address
                    .as_account_hash()
                    .unwrap_or_else(|| panic!("Address {:?} is not an account address", address))
            )
        );
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": QUERY_BALANCE_METHOD,
                "params": query_balance_params,
                "id": 1,
            }
        );
        let result: QueryBalanceResult = self.post_request(request).await;
        result.balance
    }

    pub async fn transfer(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> OdraResult<()> {
        let session = ExecutableDeployItem::Transfer {
            args: runtime_args! {
                "amount" => amount,
                "target" => to,
                "id" => Some(0u64),
            }
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        // TODO: wait_for_deploy_hash should return a result not panic, then this function can return a result
        self.wait_for_deploy_hash(deploy_hash).await;
        Ok(())
    }

    /// Returns the current block_time
    pub async fn get_block_time(&self) -> u64 {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "info_get_status",
                "id": 1,
            }
        );
        let result: Value = self.post_request(request).await;
        let result = result["last_added_block_info"]["timestamp"]
            .as_str()
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't get block time - malformed JSON response: {:?}",
                    result
                )
            });
        let system_time = humantime::parse_rfc3339_weak(result).expect("Couldn't parse block time");
        system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| panic!("Couldn't parse block time"))
            .as_millis() as u64
    }

    /// Get the event bytes from storage
    pub async fn get_event(&self, contract_address: &Address, index: u32) -> OdraResult<Bytes> {
        self.query_dict(contract_address, "__events".to_string(), index.to_string())
            .await
    }

    /// Get the events count from storage
    pub async fn events_count(&self, contract_address: &Address) -> Option<u32> {
        match self
            .query_contract_named_key(contract_address, "__events_length")
            .await
        {
            Ok(uref) => self.query_uref(uref).await.ok(),
            Err(_) => None
        }
    }

    /// Query the node for the current state root hash.
    async fn get_state_root_hash(&self) -> Digest {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "chain_get_state_root_hash",
                "id": 1,
            }
        );
        let result: GetStateRootHashResult = self.post_request(request).await;
        result.state_root_hash.unwrap()
    }

    /// Query the node for the dictionary item of a contract.
    async fn query_dict<T: FromBytes + CLTyped>(
        &self,
        contract_address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> OdraResult<T> {
        let state_root_hash = self.get_state_root_hash().await;
        let contract_hash = self
            .query_global_state_for_contract_hash(contract_address)
            .await
            .map_err(|_| OdraError::ExecutionError(ExecutionError::TypeMismatch))?;
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
        let result: GetDictionaryItemResult = self.post_request(request).await;
        match result.stored_value {
            CLValue(value) => value
                .into_t()
                .map_err(|_| OdraError::ExecutionError(ExecutionError::UnwrapError)),
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    async fn query_dict_bytes(
        &self,
        contract_address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> OdraResult<Bytes> {
        let state_root_hash = self.get_state_root_hash().await;
        let contract_hash = self
            .query_global_state_for_contract_hash(contract_address)
            .await
            .map_err(|_| OdraError::ExecutionError(ExecutionError::TypeMismatch))?;
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
        let result: Option<GetDictionaryItemResult> = self.safe_post_request(request).await;
        if let Some(result) = result {
            match result.stored_value {
                CLValue(value) => Ok(value.inner_bytes().as_slice().into()),
                _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
            }
        } else {
            Err(OdraError::ExecutionError(ExecutionError::KeyNotFound))
        }
    }

    /// Query the contract for the direct value of a named key
    pub async fn query_contract_named_key<T: AsRef<str>>(
        &self,
        contract_address: &Address,
        key: T
    ) -> Result<URef> {
        let contract_state = self
            .query_global_state_path(contract_address, key.as_ref().to_string())
            .await?;
        let uref_str = match contract_state.stored_value.clone() {
            Contract(contract) => contract.named_keys().find_map(|named_key| {
                if named_key.name == key.as_ref() {
                    Some(named_key.key.clone())
                } else {
                    None
                }
            }),
            _ => panic!("Not a contract")
        }
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Couldn't get named key {} from contract state at address {:?}",
                key.as_ref(),
                contract_address
            )
        })?;
        URef::from_formatted_str(&uref_str).map_err(|_| anyhow::anyhow!("Invalid URef format"))
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
        self.post_request(request).await
    }

    /// Discover the contract address by name.
    async fn get_contract_address(&self, key_name: &str) -> Address {
        let key_name = format!("{}_package_hash", key_name);
        let account_hash = self.public_key().to_account_hash();
        let result = self
            .query_global_state(&CasperKey::Account(account_hash))
            .await;
        let key = match result.stored_value {
            Account(account) => account
                .named_keys()
                .find(|named_key| named_key.name == key_name)
                .map_or_else(
                    || {
                        panic!(
                            "Couldn't get contract address from account state at key {}",
                            key_name
                        )
                    },
                    |named_key| named_key.key.clone()
                ),
            _ => panic!(
                "Couldn't get contract address from account state at key {}",
                key_name
            )
        }
        .as_str()
        .replace("hash-", "contract-package-wasm");
        let contract_hash = ContractPackageHash::from_formatted_str(&key).unwrap();
        Address::try_from(contract_hash).unwrap()
    }

    /// Find the contract hash by the contract package hash.
    async fn query_global_state_for_contract_hash(
        &self,
        address: &Address
    ) -> Result<ContractHash> {
        let contract_package_hash = address
            .as_contract_package_hash()
            .context("Not a contract package hash")?;
        let key = CasperKey::Hash(contract_package_hash.value());
        let result = self.query_global_state(&key).await;
        let result_as_json = serde_json::to_value(result).context("Couldn't parse result")?;
        let contract_hash: &str = result_as_json["stored_value"]["ContractPackage"]["versions"][0]
            ["contract_hash"]
            .as_str()
            .context("Couldn't get contract hash")?;
        ContractHash::from_formatted_str(contract_hash)
            .map_err(|_| anyhow::anyhow!("Invalid contract hash format"))
    }

    /// Deploy the contract.
    pub async fn deploy_wasm(
        &mut self,
        contract_name: &str,
        args: RuntimeArgs,
        timestamp: Timestamp
    ) -> OdraResult<Address> {
        log::info(format!("Deploying \"{}\".", contract_name));
        let wasm_path = find_wasm_file_path(contract_name);
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::from(wasm_bytes),
            args
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy_hash(deploy_hash).await;
        self.process_result(
            result,
            ContractId::Name(contract_name.to_string()),
            deploy_hash
        )?;

        let address = self.get_contract_address(contract_name).await;
        log::info(format!("Contract {:?} deployed.", &address.to_string()));
        Ok(address)
    }

    pub fn register_name(&mut self, address: Address, contract_name: String) {
        self.contracts.insert(address, contract_name);
    }

    fn find_error(&self, contract_id: ContractId, error_msg: &str) -> Option<(String, OdraError)> {
        match contract_id {
            ContractId::Name(contract_name) => error::find(&contract_name, error_msg).ok(),
            ContractId::Address(addr) => match self.contracts.get(&addr) {
                Some(contract_name) => error::find(contract_name, error_msg).ok(),
                None => None
            }
        }
    }

    /// Deploy the entrypoint call using getter_proxy.
    /// It runs the getter_proxy contract in an account context and stores the return value of the call
    /// in under the key RESULT_KEY.
    pub async fn deploy_entrypoint_call_with_proxy(
        &self,
        addr: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> OdraResult<Bytes> {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\" through proxy.",
            addr.to_string(),
            call_def.entry_point()
        ));

        let args = runtime_args! {
            CONTRACT_PACKAGE_HASH_ARG => *addr.as_contract_package_hash().unwrap(),
            ENTRY_POINT_ARG => call_def.entry_point(),
            ARGS_ARG => Bytes::from(call_def.args().to_bytes().unwrap()),
            ATTACHED_VALUE_ARG => call_def.amount(),
            AMOUNT_ARG => call_def.amount(),
        };

        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: include_bytes!("../resources/proxy_caller_with_return.wasm")
                .to_vec()
                .into(),
            args
        };

        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy_hash(deploy_hash).await;
        self.process_result(result, ContractId::Address(addr), deploy_hash)?;

        let r = self
            .query_global_state(&CasperKey::Account(self.public_key().to_account_hash()))
            .await;
        match r.stored_value {
            Account(account) => {
                let result = account
                    .named_keys()
                    .find(|named_key| named_key.name == RESULT_KEY);
                match result {
                    Some(result) => {
                        self.query_uref(URef::from_formatted_str(&result.key).unwrap())
                            .await
                    }
                    None => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
                }
            }
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    /// Deploy the entrypoint call.
    pub async fn deploy_entrypoint_call(
        &self,
        addr: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> OdraResult<Bytes> {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\".",
            addr.to_string(),
            call_def.entry_point()
        ));
        let session = ExecutableDeployItem::StoredVersionedContractByHash {
            hash: *addr.as_contract_package_hash().unwrap(),
            version: None,
            entry_point: call_def.entry_point().to_string(),
            args: call_def.args().clone()
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy_hash(deploy_hash).await;

        self.process_result(result, ContractId::Address(addr), deploy_hash)
            .map(|_| ().to_bytes().expect("Couldn't serialize (). This shouldn't happen.").into())
    }

    async fn query_global_state_path(
        &self,
        address: &Address,
        _path: String
    ) -> Result<QueryGlobalStateResult> {
        let hash = self.query_global_state_for_contract_hash(address).await?;
        let key = CasperKey::Hash(hash.value());
        let state_root_hash = self.get_state_root_hash().await;
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
        Ok(self.post_request(request).await)
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
        self.post_request(request).await
    }

    async fn query_state_dictionary(&self, address: &Address, key: &str) -> Result<Bytes> {
        let state_root_hash = self.get_state_root_hash().await;
        let contract_hash = self.query_global_state_for_contract_hash(address).await?;
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
        let result: Option<GetDictionaryItemResult> = self.post_request(request).await;
        result
            .context("Couldn't get dictionary item")
            .and_then(|result| {
                let result_as_json =
                    serde_json::to_value(result).context("Couldn't parse result")?;
                let result = result_as_json["stored_value"]["CLValue"]["bytes"]
                    .as_str()
                    .context("Couldn't get bytes")?;
                let bytes = hex::decode(result).context("Couldn't decode bytes")?;
                let (value, _) = FromBytes::from_bytes(&bytes)
                    .map_err(|_| anyhow::anyhow!("Couldn't parse bytes"))?;

                Ok(value)
            })
    }

    async fn query_uref<T: CLTyped + FromBytes>(&self, uref: URef) -> OdraResult<T> {
        let result = self.query_global_state(&CasperKey::URef(uref)).await;
        match result.stored_value {
            CLValue(value) => value
                .into_t()
                .map_err(|_| OdraError::ExecutionError(ExecutionError::UnwrapError)),
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    async fn query_uref_bytes(&self, uref: URef) -> OdraResult<Bytes> {
        let key = CasperKey::URef(uref);
        let result = self.query_global_state(&key).await;
        match result.stored_value {
            CLValue(value) => Ok(value.inner_bytes().as_slice().into()),
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    async fn wait_for_deploy_hash(&self, deploy_hash: DeployHash) -> ExecutionResult {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let time_diff = Duration::from_secs(15);
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &time_diff, &deploy_hash_str
            ));
            sleep(time_diff).await;
            let result: GetDeployResult = self.get_deploy(deploy_hash).await;
            if !result.execution_results.is_empty() {
                final_result = result;
                break;
            }
        }
        final_result.execution_results[0].result.clone()
    }

    fn process_result(
        &self,
        result: ExecutionResult,
        called_contract_id: ContractId,
        deploy_hash: DeployHash
    ) -> OdraResult<()> {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        match result {
            ExecutionResult::Failure { error_message, .. } => {
                let (error_msg, odra_error) =
                    match self.find_error(called_contract_id, &error_message) {
                        Some((contract_error, odra_error)) => (contract_error, odra_error),
                        None => (
                            error_message,
                            OdraError::ExecutionError(ExecutionError::UnexpectedError)
                        )
                    };
                log::error(format!(
                    "Deploy {:?} failed with error: {:?}.",
                    deploy_hash_str, error_msg
                ));
                Err(odra_error)
            }
            ExecutionResult::Success { .. } => {
                log::info(format!(
                    "Deploy {:?} successfully executed.",
                    deploy_hash_str
                ));
                Ok(())
            }
        }
    }

    fn new_deploy(&self, session: ExecutableDeployItem, gas: U512, timestamp: Timestamp) -> Deploy {
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
            self.secret_key(),
            Some(self.public_key())
        )
    }

    async fn safe_post_request<T: DeserializeOwned>(&self, request: Value) -> Option<T> {
        let client = reqwest::Client::new();

        let mut client = client.post(self.node_address_rpc());
        if let Some(token) = &self.configuration.cspr_cloud_auth_token {
            client = client.header("Authorization", token);
        }
        let response = client.json(&request).send().await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                log::error(format!("Couldn't send request: {:?}", e));
                panic!("Couldn't send request")
            }
        };
        let json: JsonRpc = response.json().await.unwrap_or_else(|e| {
            log::error(format!("Couldn't parse response: {:?}", e));
            panic!("Couldn't parse response")
        });
        json.get_result()
            .map(|result| {
                serde_json::from_value::<T>(result.clone()).unwrap_or_else(|e| {
                    log::error(format!("Couldn't parse result: {:?}", e));
                    panic!("Couldn't parse result")
                })
            })
    }

    async fn post_request<T: DeserializeOwned>(&self, request: Value) -> T {
        self.safe_post_request(request)
            .await
            .unwrap_or_else(|| {
                log::error(format!("Couldn't get result: {:?}", json));
                panic!("Couldn't get result")
            })
    }

    fn address_secret_key(&self, address: &Address) -> &SecretKey {
        match self
            .secret_keys()
            .iter()
            .find(|key| Address::from(PublicKey::from(*key)) == *address)
        {
            Some(secret_key) => secret_key,
            None => panic!("Key for address {:?} is not loaded", address)
        }
    }

    fn secret_keys(&self) -> &Vec<SecretKey> {
        &self.configuration.secret_keys
    }
}

impl Default for CasperClient {
    fn default() -> Self {
        Self::new(CasperClientConfiguration::from_env())
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

fn put_deploy_request(deploy: Deploy) -> Value {
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
    request
}
