//! Client for interacting with Casper node.

use std::collections::BTreeMap;
use std::time::Duration;
use std::time::SystemTime;

use itertools::Itertools;
use jsonrpc_lite::JsonRpc;
use odra_core::OdraResult;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::casper_client::configuration::CasperClientConfiguration;

use crate::log;
use anyhow::{Context, Result};
use casper_client::cli::{
    get_balance, get_deploy, get_dictionary_item, get_entity, get_state_root_hash,
    query_global_state, DictionaryItemStrParams
};
use casper_client::rpcs::results::{GetDeployResult, GetDictionaryItemResult, PutDeployResult};
use casper_client::rpcs::DictionaryItemIdentifier;
use casper_client::Verbosity;
use casper_types::bytesrepr::deserialize_from_slice;
use casper_types::execution::ExecutionResultV1::{Failure, Success};
use casper_types::StoredValue::CLValue;
use casper_types::{execution::ExecutionResult, EntityAddr};
use casper_types::{Deploy, DeployHash, ExecutableDeployItem, StoredValue, TimeDiff, Timestamp};
use odra_core::casper_types::{sign, URef};
use odra_core::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        runtime_args, CLTyped, PublicKey, RuntimeArgs, SecretKey, U512
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
/// Environment variable holding the URL of the events stream.
pub const ENV_EVENTS_ADDRESS: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
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
        Ok(self
            .get_dictionary_value(address, "state", key)
            .await
            .unwrap())
    }

    /// Gets a value from a named key
    pub async fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        let entity_hash = self
            .query_global_state_for_entity_addr(address)
            .await
            .unwrap();
        let stored_value = self
            .query_global_state(&entity_hash.to_formatted_string(), Some(name.to_string()))
            .await;
        match stored_value.clone() {
            CLValue(value) => Some(Bytes::from(value.inner_bytes().as_slice())),
            _ => {
                panic!(
                    "Couldn't get {} from {:?}",
                    name,
                    address.to_formatted_string()
                )
            }
        }
    }

    /// Gets a value from a result key
    pub async fn get_proxy_result(&self) -> Bytes {
        let stored_value = self
            .query_global_state(
                &self
                    .caller()
                    .as_account_hash()
                    .unwrap()
                    .to_formatted_string(),
                Some(RESULT_KEY.to_string())
            )
            .await;
        match stored_value {
            CLValue(value) => {
                let bytes: Bytes = value.into_t().unwrap();
                bytes
            }
            _ => panic!("Value stored in result key is not a CLValue")
        }
    }

    /// Gets a value from a named dictionary
    pub async fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        let key = String::from_utf8(key.to_vec()).unwrap();
        self.query_dict(address, dictionary_name.to_string(), key)
            .await
            .ok()
    }

    /// Sets amount of gas for the next deploy.
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas.into();
    }

    /// Node rpc address.
    pub fn node_address_rpc(&self) -> String {
        format!("{}/rpc", self.configuration.node_address)
    }

    /// Node address.
    pub fn node_address(&self) -> &str {
        &self.configuration.node_address
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
        // TODO: Use rpc when it will be public to do this in one call
        let main_purse = self.get_main_purse(address).await.to_formatted_string();
        get_balance(
            // TODO: set rpc id to a random number
            "",
            &self.configuration.node_address,
            self.configuration.verbosity(),
            &self.get_state_root_hash().await,
            &main_purse
        )
        .await
        .unwrap()
        .result
        .balance_value
    }

    pub async fn get_main_purse(&self, address: &Address) -> URef {
        let purse_uref = self
            .query_global_state(&address.to_formatted_string(), None)
            .await;
        match purse_uref {
            CLValue(value) => {
                value.into_t().unwrap()
            }
            StoredValue::AddressableEntity(entity) => entity.main_purse(),
            _ => panic!("Not an addressable entity")
        }
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
        self.wait_for_deploy(deploy_hash).await;
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
        self.get_named_value(contract_address, "__events_length")
            .await
            .map(|bytes| deserialize_from_slice(&bytes).unwrap())
    }

    /// Query the node for the current state root hash.
    pub async fn get_state_root_hash(&self) -> String {
        let digest = get_state_root_hash(
            "",
            &self.configuration.node_address,
            Verbosity::Low as u64,
            ""
        )
        .await
        .unwrap()
        .result
        .state_root_hash
        .unwrap();

        base16::encode_lower(&digest)
    }

    /// Query the node for the dictionary item of a contract or an account.
    async fn query_dict<T: FromBytes + CLTyped>(
        &self,
        address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> OdraResult<T> {
        let entity_addr = self
            .query_global_state_for_entity_addr(address)
            .await
            .unwrap();
        let params = DictionaryItemStrParams::EntityNamedKey {
            entity_addr: &entity_addr.to_formatted_string(),
            dictionary_name: &dictionary_name,
            dictionary_item_key: &dictionary_item_key
        };
        let r = get_dictionary_item(
            "",
            &self.configuration.node_address,
            Verbosity::Low as u64,
            &self.get_state_root_hash().await,
            params
        )
        .await;

        r.unwrap()
            .result
            .stored_value
            .into_cl_value()
            .unwrap()
            .into_t()
            .map_err(|_| OdraError::ExecutionError(ExecutionError::TypeMismatch))
    }

    /// Query the node for the transaction state.
    pub async fn get_deploy(&self, deploy_hash: DeployHash) -> GetDeployResult {
        let t = get_deploy(
            "",
            &self.configuration.node_address.clone(),
            Verbosity::Low as u64,
            &deploy_hash.to_hex_string(),
            true
        )
        .await;

        t.unwrap().result
    }

    /// Discover the contract address by name.
    async fn get_contract_address(&self, key_name: &str) -> Address {
        let key_name = format!("{}_package_hash", key_name);

        // TODO: set rpc id to a random number
        let result = get_entity(
            "",
            &self.configuration.node_address,
            Verbosity::Low as u64,
            "",
            &self.public_key().to_hex_string()
        )
        .await
        .unwrap()
        .result;
        let account = result.entity_result.addressable_entity().unwrap();

        let key = account.named_keys.get(&key_name).unwrap();

        Address::from(key.into_package_hash().unwrap())
    }

    /// Find the entity addr in global state for an address
    async fn query_global_state_for_entity_addr(&self, address: &Address) -> Result<EntityAddr> {
        let result = self
            .query_global_state(&address.to_formatted_string(), None)
            .await;
        match result {
            StoredValue::Package(package) => Ok(EntityAddr::SmartContract(
                package.current_entity_hash().unwrap().value()
            )),
            _ => Err(anyhow::anyhow!("Not a package"))
        }
    }

    /// Deploy the contract.
    pub async fn deploy_wasm(
        &mut self,
        contract_name: &str,
        args: RuntimeArgs,
        timestamp: Timestamp,
        wasm_bytes: Vec<u8>
    ) -> OdraResult<Address> {
        log::info(format!("Deploying \"{}\".", contract_name));
        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::from(wasm_bytes),
            args
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy(deploy_hash).await;
        self.process_execution(
            result,
            ContractId::Name(contract_name.to_string()),
            deploy_hash
        )?;

        let address = self.get_contract_address(contract_name).await;
        log::info(format!(
            "Contract {:?} deployed.",
            &address.to_formatted_string()
        ));

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
        address: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> OdraResult<Bytes> {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\" through proxy.",
            address.to_formatted_string(),
            call_def.entry_point()
        ));

        let hash = address.as_package_hash().unwrap();
        let args_bytes: Vec<u8> = call_def
            .args()
            .to_bytes()
            .expect("Should serialize to bytes");
        let entry_point = call_def.entry_point();
        let args = runtime_args! {
            PACKAGE_HASH_ARG => hash,
            ENTRY_POINT_ARG => entry_point,
            ARGS_ARG => Bytes::from(args_bytes),
            ATTACHED_VALUE_ARG => call_def.amount(),
            AMOUNT_ARG => call_def.amount(),
        };

        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: include_bytes!("../../test-vm/resources/proxy_caller_with_return.wasm")
                .to_vec()
                .into(),
            args
        };

        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy(deploy_hash).await;
        self.process_execution(result, ContractId::Address(address), deploy_hash)?;
        Ok(self.get_proxy_result().await)
    }

    /// Deploy the entrypoint call.
    pub async fn deploy_entrypoint_call(
        &self,
        addr: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> OdraResult<Bytes> {
        log::info(format!(
            "Calling {:?} directly with entrypoint \"{}\".",
            addr.to_formatted_string(),
            call_def.entry_point()
        ));
        let session = ExecutableDeployItem::StoredVersionedContractByHash {
            hash: addr.as_package_hash().unwrap(),
            version: None,
            entry_point: call_def.entry_point().to_string(),
            args: call_def.args().clone()
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy(deploy_hash).await;

        self.process_execution(result, ContractId::Address(addr), deploy_hash)
            .map(|_| ().to_bytes().expect("Couldn't serialize (). This shouldn't happen.").into())
    }

    async fn query_global_state(&self, key: &str, path: Option<String>) -> StoredValue {
        // Todo: set rpc id to a random number
        query_global_state(
            "",
            &self.configuration.node_address,
            Verbosity::Low as u64,
            "",
            &self.get_state_root_hash().await,
            key,
            &path.unwrap_or_default()
        )
        .await
        .unwrap()
        .result
        .stored_value
    }

    async fn _query_state_dictionary(&self, address: &Address, key: &str) -> Result<Bytes> {
        let contract_hash = self.query_global_state_for_entity_addr(address).await?;
        let contract_hash = contract_hash
            .to_formatted_string()
            .replace("contract-", "hash-");
        let params = DictionaryItemIdentifier::ContractNamedKey {
            key: contract_hash,
            dictionary_name: "state".to_string(),
            dictionary_item_key: key.to_string()
        };
        // TODO: set rpc id to a random number
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "state_get_dictionary_item",
                "params": params,
                "id": 1,
            }
        );

        let result = self
            .safe_post_request(request.clone())
            .await
            .get_result()
            .and_then(|result| {
                serde_json::from_value::<GetDictionaryItemResult>(result.clone()).ok()
            });
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

    async fn _query_uref<T: CLTyped + FromBytes>(&self, _uref: URef) -> OdraResult<T> {
        todo!()
        // let result = self.query_global_state(&CasperKey::URef(uref)).await;
        // match result.stored_value {
        //     CLValue(value) => value
        //         .into_t()
        //         .map_err(|_| OdraError::ExecutionError(ExecutionError::UnwrapError)),
        //     _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        // }
    }

    async fn _query_uref_bytes(&self, _uref: URef) -> OdraResult<Bytes> {
        todo!()
        // let key = CasperKey::URef(uref);
        // let result = self.query_global_state(&key).await;
        // match result.stored_value {
        //     CLValue(value) => Ok(value.inner_bytes().as_slice().into()),
        //     _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        // }
    }

    async fn wait_for_deploy(&self, deploy_hash: DeployHash) -> ExecutionResult {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let time_diff = Duration::from_secs(15);
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &time_diff, &deploy_hash_str
            ));
            sleep(time_diff).await;
            let result = self.get_deploy(deploy_hash).await.execution_info.unwrap();
            if result.execution_result.is_some() {
                final_result = result.execution_result.unwrap();
                break;
            }
        }
        final_result.clone()
    }

    fn process_execution(
        &self,
        result: ExecutionResult,
        called_contract_id: ContractId,
        deploy_hash: DeployHash
    ) -> OdraResult<()> {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        match result {
            ExecutionResult::V1(r) => match r {
                Failure { error_message, .. } => {
                    let (error_msg, odra_error) =
                        match self.find_error(called_contract_id, &error_message) {
                            Some((contract_error, odra_error)) => (contract_error, odra_error),
                            None => (
                                error_message,
                                OdraError::ExecutionError(ExecutionError::UnexpectedError)
                            )
                        };
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_msg
                    ));
                    Err(odra_error)
                }
                Success { .. } => {
                    log::info(format!(
                        "Deploy {:?} successfully executed.",
                        deploy_hash_str
                    ));
                    Ok(())
                }
            },
            ExecutionResult::V2(r) => match r.error_message {
                None => {
                    log::info(format!(
                        "Deploy {:?} successfully executed.",
                        deploy_hash_str
                    ));
                    Ok(())
                }
                Some(e) => {
                    log::error(format!(
                        "Deploy V2 {:?} failed with error: {:?}.",
                        deploy_hash_str, e
                    ));
                    Err(OdraError::ExecutionError(ExecutionError::UnexpectedError))
                }
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

    async fn safe_post_request(&self, request: Value) -> JsonRpc {
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
        json
    }

    async fn post_request<T: DeserializeOwned>(&self, request: Value) -> T {
        let json = self.safe_post_request(request.clone()).await;
        json.get_result()
            .map(|result| {
                serde_json::from_value::<T>(result.clone()).unwrap_or_else(|e| {
                    log::error(format!("Couldn't parse result: {:?}", e));
                    panic!("Couldn't parse result")
                })
            })
            .unwrap_or_else(|| {
                log::error(format!("Couldn't get result: {:?} - {:?}", json, request));
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

#[cfg(feature = "std")]
impl Default for CasperClient {
    fn default() -> Self {
        Self::new(CasperClientConfiguration::from_env())
    }
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
