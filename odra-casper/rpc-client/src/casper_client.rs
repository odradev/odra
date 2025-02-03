//! Client for interacting with Casper node.

use itertools::Itertools;
use jsonrpc_lite::JsonRpc;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::casper_client::configuration::CasperClientConfiguration;

use crate::error::Error;
use crate::error::Error::{Execution, LivenetToDo};
use crate::log;
use casper_client::cli::{
    get_dictionary_item, get_entity, get_node_status, get_state_root_hash, DictionaryItemStrParams
};
use casper_client::rpcs::results::{GetDeployResult, GetTransactionResult, PutDeployResult};
use casper_client::rpcs::GlobalStateIdentifier;
use casper_client::{
    get_balance, get_deploy, get_transaction, put_transaction, query_global_state, JsonRpcId,
    Verbosity
};
use casper_types::bytesrepr::{deserialize_from_slice, Bytes, ToBytes};
use casper_types::contracts::ContractPackageHash;
use casper_types::execution::ExecutionResultV1::{Failure, Success};
use casper_types::StoredValue::CLValue;
use casper_types::{
    execution::ExecutionResult, runtime_args, sign, CLTyped, Digest, EntityAddr, Key, PublicKey,
    RuntimeArgs, SecretKey, Transaction, TransactionHash, URef, U512
};
use casper_types::{Deploy, DeployHash, ExecutableDeployItem, StoredValue, TimeDiff, Timestamp};
use odra_core::casper_event_standard::EVENTS_LENGTH;
use odra_core::consts::{
    AMOUNT_ARG, ARGS_ARG, ATTACHED_VALUE_ARG, ENTRY_POINT_ARG, EVENTS, PACKAGE_HASH_ARG,
    RESULT_KEY, STATE_KEY
};
use odra_core::prelude::*;
use odra_core::CallDef;

pub mod configuration;

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
/// Time between retries when waiting for a deploy to be processed.
pub const DEPLOY_WAIT_TIME: u64 = 10;

pub type Result<T> = core::result::Result<T, Error>;

/// Client for interacting with Casper node.
pub struct CasperClient {
    configuration: CasperClientConfiguration,
    active_account: usize,
    gas: U512
}

impl CasperClient {
    /// Creates new CasperClient.
    pub fn new(configuration: CasperClientConfiguration) -> Self {
        CasperClient {
            configuration,
            active_account: 0,
            gas: U512::zero()
        }
    }

    /// Gets a value from the Odra storage (`state` dictionary)
    pub async fn get_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.get_dictionary_value(address, STATE_KEY, key).await
    }

    /// Gets a value from a named key of an account or a contract
    pub async fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        let entity_hash = self.query_global_state_for_entity_addr(address).await;
        let stored_value = self
            .query_global_state(Key::Hash(entity_hash.value()), Some(name.to_string()))
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
            .query_global_state(self.caller().as_key(), Some(RESULT_KEY.to_string()))
            .await;
        match stored_value {
            CLValue(value) => value
                .clone()
                .into_t()
                .unwrap_or_else(|_| panic!("Couldn't get bytes from CLValue: {:?}", value)),
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
        let key = String::from_utf8(key.to_vec())
            .unwrap_or_else(|_| panic!("Couldn't convert key to string: {:?}", key));
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
        format!("{}/rpc", self.node_address())
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
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Result<Bytes> {
        let secret_key = self.address_secret_key(address);
        let public_key = &PublicKey::from(secret_key);
        let signature = sign(message, secret_key, public_key)
            .to_bytes()
            .map_err(|_| LivenetToDo)?;

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
        let main_purse = self.get_main_purse(address);
        get_balance(
            self.rpc_id_typed(),
            self.node_address(),
            self.configuration.verbosity_typed(),
            self.get_state_root_hash_digest().await,
            main_purse.await
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Couldn't get balance for address: {:?}",
                address.to_formatted_string()
            )
        })
        .result
        .balance_value
    }

    /// Gets an uref of a main purse of an account or a contract.
    pub async fn get_main_purse(&self, address: &Address) -> URef {
        let purse_uref = self.query_global_state(address.as_key(), None).await;
        match purse_uref {
            CLValue(value) => value.into_t().unwrap(),
            StoredValue::AddressableEntity(entity) => entity.main_purse(),
            StoredValue::Account(account) => account.main_purse(),
            _ => panic!("Getting main purse is not supported for: {:?}", purse_uref)
        }
    }

    pub async fn transfer(&self, to: Address, amount: U512, timestamp: Timestamp) -> Result<()> {
        let session = ExecutableDeployItem::Transfer {
            args: runtime_args! {
                "amount" => amount,
                "target" => to,
                "id" => Some(0u64),
            }
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await?;
        let deploy_hash = response.deploy_hash;
        // TODO: wait_for_deploy_hash should return a result not panic, then this function can return a result
        self.wait_for_deploy(deploy_hash).await?;
        Ok(())
    }

    /// Returns the current block_time
    pub async fn get_block_time(&self) -> Result<u64> {
        let block_time = get_node_status(
            &self.rpc_id(),
            self.node_address(),
            self.configuration.verbosity()
        )
        .await
        .map_err(|_| LivenetToDo)?
        .result
        .last_added_block_info
        .ok_or(LivenetToDo)?
        .timestamp
        .millis();
        Ok(block_time)
    }

    /// Get the event bytes from storage
    pub async fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes> {
        self.query_dict(contract_address, EVENTS.to_string(), index.to_string())
            .await
    }

    /// Get the events count from storage
    pub async fn events_count(&self, contract_address: &Address) -> Option<u32> {
        self.get_named_value(contract_address, EVENTS_LENGTH)
            .await
            .map(|bytes| {
                deserialize_from_slice(&bytes).unwrap_or_else(|_| {
                    panic!(
                        "Couldn't deserialize events count for contract: {:?}, bytes: {:?}",
                        contract_address, bytes
                    )
                })
            })
    }

    /// Query the node for the current state root hash.
    pub async fn get_state_root_hash(&self) -> String {
        base16::encode_lower(&self.get_state_root_hash_digest().await)
    }

    pub async fn get_state_root_hash_digest(&self) -> Digest {
        get_state_root_hash(
            &self.rpc_id(),
            self.node_address(),
            self.configuration.verbosity(),
            ""
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Couldn't get state root hash from node: {:?}",
                self.node_address()
            )
        })
        .result
        .state_root_hash
        .unwrap_or_else(|| {
            panic!(
                "Couldn't get state root hash from node: {:?}",
                self.node_address()
            )
        })
    }

    /// Query the node for the dictionary item of a contract or an account.
    /// TODO: consider remove T
    async fn query_dict(
        &self,
        address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<Bytes> {
        let entity_addr = self.query_global_state_for_entity_addr(address).await;
        let hash_addr = Key::Hash(entity_addr.value()).to_formatted_string();
        let params = DictionaryItemStrParams::ContractNamedKey {
            hash_addr: &hash_addr,
            dictionary_name: &dictionary_name,
            dictionary_item_key: &dictionary_item_key
        };
        println!("hash_addr: {:?}", hash_addr);
        println!("dict_name: {:?}", dictionary_name);
        println!("dict_item_key: {:?}", dictionary_item_key);

        let r = get_dictionary_item(
            &self.rpc_id(),
            self.node_address(),
            self.configuration.verbosity(),
            &self.get_state_root_hash().await,
            params
        )
        .await;

        let result = r.map_err(|_| LivenetToDo)?;
        let stored_value = result.result.stored_value;
        let cl_value = stored_value.into_cl_value().ok_or(LivenetToDo)?;

        // Note: this is for compatibility with CEP18 named keys.
        if cl_value.cl_type() == &Vec::<u8>::cl_type() {
            let bytes = cl_value.into_t().map_err(|_| LivenetToDo)?;
            Ok(bytes)
        } else {
            let bytes = cl_value.inner_bytes();
            Ok(Bytes::from(bytes.to_vec()))
        }
    }

    /// Query the node for the transaction state.
    pub async fn get_transaction(&self, transaction_hash: TransactionHash) -> GetTransactionResult {
        let t = get_transaction(
            self.rpc_id_typed(),
            self.node_address(),
            // self.configuration.verbosity_typed(),
            Verbosity::High,
            transaction_hash,
            true
        )
        .await;
        t.unwrap_or_else(|e| {
            log::error(format!("Couldn't get transaction: {:?}", e));
            panic!(
                "Couldn't get transaction: {:?}",
                transaction_hash.to_hex_string().as_str()
            )
        })
        .result
    }

    /// Query the node for the transaction state.
    pub async fn get_deploy(&self, deploy_hash: DeployHash) -> GetDeployResult {
        let t = get_deploy(
            self.rpc_id_typed(),
            self.node_address(),
            // self.configuration.verbosity_typed(),
            Verbosity::High,
            deploy_hash,
            true
        )
        .await;
        t.unwrap_or_else(|e| {
            log::error(format!("Couldn't get deploy: {:?}", e));
            panic!(
                "Couldn't get deploy: {:?}",
                deploy_hash.to_hex_string().as_str()
            )
        })
        .result
    }

    /// Discover the contract address by name.
    async fn get_contract_address(&self, key_name: &str) -> Address {
        let key_name = format!("{}_{}", key_name, PACKAGE_HASH_ARG);

        let result = get_entity(
            &self.rpc_id(),
            self.node_address(),
            self.configuration.verbosity(),
            "",
            &self.public_key().to_hex_string()
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "{}",
                format!(
                    "Couldn't get entity for public key: {:?}",
                    &self.public_key().to_hex_string()
                )
            );
        })
        .result;
        let account = result
            .entity_result
            .addressable_entity()
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't get addressable entity for public key: {:?}",
                    self.public_key().to_hex_string()
                )
            });

        let key = account.named_keys.get(&key_name).unwrap_or_else(|| {
            panic!(
                "Couldn't get named key {:?} for account: {:?}",
                key_name,
                self.public_key().to_hex_string()
            )
        });

        Address::from(key.into_package_hash().unwrap_or_else(|| {
            panic!(
                "Couldn't get package hash from key {:?} for account: {:?}",
                key_name,
                self.public_key().to_hex_string()
            )
        }))
    }

    /// Find the entity addr in global state for an address
    /// TODO: Remove this method.
    async fn query_global_state_for_entity_addr(&self, address: &Address) -> EntityAddr {
        let result = self.query_global_state(address.as_key(), None).await;
        match result {
            StoredValue::SmartContract(package) => EntityAddr::SmartContract(
                package
                    .current_entity_hash()
                    .unwrap_or_else(|| {
                        panic!(
                            "Couldn't get entity addr for address: {:?}",
                            address.to_formatted_string()
                        )
                    })
                    .value()
            ),
            StoredValue::ContractPackage(package) => {
                let last_version = package.current_contract_hash().unwrap();
                EntityAddr::SmartContract(last_version.value())
            }
            _ => {
                panic!(
                    "Couldn't get entity addr for address: {:?}",
                    address.to_formatted_string()
                )
            }
        }
    }

    /// Deploy the contract.
    pub async fn deploy_wasm(
        &mut self,
        contract_name: &str,
        args: RuntimeArgs,
        timestamp: Timestamp,
        wasm_bytes: Vec<u8>
    ) -> Result<Address> {
        log::info(format!("Deploying \"{}\".", contract_name));
        let session = ExecutableDeployItem::ModuleBytes {
            module_bytes: Bytes::from(wasm_bytes),
            args
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let response = put_transaction(
            self.rpc_id_typed(),
            self.node_address(),
            self.configuration.verbosity_typed(),
            Transaction::Deploy(deploy)
        )
        .await;
        let deploy_hash = response.unwrap().result.transaction_hash;
        let result = self.wait_for_transaction(deploy_hash).await?;
        self.process_transaction(result, deploy_hash)?;

        let address = self.get_contract_address(contract_name).await;
        log::info(format!(
            "Contract {:?} deployed.",
            &address.to_formatted_string()
        ));

        Ok(address)
    }

    /// Deploy the entrypoint call using getter_proxy.
    /// It runs the getter_proxy contract in an account context and stores the return value of the call
    /// in under the key RESULT_KEY.
    pub async fn deploy_entrypoint_call_with_proxy(
        &self,
        address: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Result<Bytes> {
        log::info(format!(
            "Calling {:?} with entrypoint \"{}\" through proxy.",
            address.to_formatted_string(),
            call_def.entry_point()
        ));

        let hash = address.as_contract_package_hash().unwrap();
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
        let response: PutDeployResult = self.post_request(request).await?;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy(deploy_hash).await?;
        self.process_execution(result, deploy_hash)?;
        Ok(self.get_proxy_result().await)
    }

    /// Deploy the entrypoint call.
    pub async fn deploy_entrypoint_call(
        &self,
        addr: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Result<Bytes> {
        log::info(format!(
            "Calling {:?} directly with entrypoint \"{}\".",
            addr.to_formatted_string(),
            call_def.entry_point()
        ));
        let session = ExecutableDeployItem::StoredVersionedContractByHash {
            hash: ContractPackageHash::from(
                addr.as_contract_package_hash()
                    .unwrap_or_else(|| {
                        panic!(
                            "Couldn't get package hash from address: {:?}",
                            addr.to_formatted_string()
                        )
                    })
                    .value()
            ),
            version: None,
            entry_point: call_def.entry_point().to_string(),
            args: call_def.args().clone()
        };
        let deploy = self.new_deploy(session, self.gas, timestamp);
        let request = put_deploy_request(deploy);
        let response: PutDeployResult = self.post_request(request).await?;
        let deploy_hash = response.deploy_hash;
        let result = self.wait_for_deploy(deploy_hash).await?;

        self.process_execution(result, deploy_hash).map(|_| {
            ().to_bytes()
                .expect("Couldn't serialize (). This shouldn't happen.")
                .into()
        })
    }

    async fn query_global_state(&self, key: Key, path: Option<String>) -> StoredValue {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        query_global_state(
            self.rpc_id_typed(),
            self.node_address(),
            self.configuration.verbosity_typed(),
            GlobalStateIdentifier::StateRootHash(self.get_state_root_hash_digest().await),
            key,
            path
        )
        .await
        .unwrap_or_else(|e| {
            log::error(format!("Couldn't query global state: {:?}", e));
            panic!("Couldn't query global state")
        })
        .result
        .stored_value
    }

    async fn wait_for_transaction(
        &self,
        transation_hash: TransactionHash
    ) -> Result<ExecutionResult> {
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &DEPLOY_WAIT_TIME, &transation_hash
            ));

            tokio::time::sleep(std::time::Duration::from_secs(DEPLOY_WAIT_TIME)).await;

            let result = self.get_transaction(transation_hash).await.execution_info;

            if result.is_some() {
                final_result = result
                    .ok_or(LivenetToDo)?
                    .execution_result
                    .ok_or(LivenetToDo)?;
                break;
            }
        }
        Ok(final_result.clone())
    }

    async fn wait_for_deploy(&self, deploy_hash: DeployHash) -> Result<ExecutionResult> {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        let final_result;

        loop {
            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &DEPLOY_WAIT_TIME, &deploy_hash_str
            ));

            tokio::time::sleep(std::time::Duration::from_secs(DEPLOY_WAIT_TIME)).await;

            let result = self.get_deploy(deploy_hash).await.execution_info;

            if result.is_some() {
                final_result = result
                    .ok_or(LivenetToDo)?
                    .execution_result
                    .ok_or(LivenetToDo)?;
                break;
            }
        }
        Ok(final_result.clone())
    }

    fn process_transaction(
        &self,
        result: ExecutionResult,
        deploy_hash: TransactionHash
    ) -> Result<()> {
        let deploy_hash_str = deploy_hash.to_hex_string();
        match result {
            ExecutionResult::V1(r) => match r {
                Failure { error_message, .. } => {
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message
                    ));
                    Err(Execution { error_message })
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
                Some(error_message) => {
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message
                    ));
                    Err(Execution { error_message })
                }
            }
        }
    }
    fn process_execution(&self, result: ExecutionResult, deploy_hash: DeployHash) -> Result<()> {
        let deploy_hash_str = format!("{:?}", deploy_hash.inner());
        match result {
            ExecutionResult::V1(r) => match r {
                Failure { error_message, .. } => {
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message
                    ));
                    Err(Execution { error_message })
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
                Some(error_message) => {
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message
                    ));
                    Err(Execution { error_message })
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

        Deploy::new_signed(
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

    async fn safe_post_request(&self, request: Value) -> Result<JsonRpc> {
        let client = reqwest::Client::new();

        let mut client = client.post(self.node_address_rpc());
        if let Some(token) = &self.configuration.cspr_cloud_auth_token {
            client = client.header("Authorization", token);
        }

        let response = client
            .json(&request)
            .send()
            .await
            .map_err(|_| LivenetToDo)?;
        let json: JsonRpc = response.json().await.map_err(|_| LivenetToDo)?;
        Ok(json)
    }

    async fn post_request<T: DeserializeOwned>(&self, request: Value) -> Result<T> {
        let json = self.safe_post_request(request.clone()).await?;
        let result = json
            .get_result()
            .map(|result| serde_json::from_value::<T>(result.clone()))
            .unwrap()
            .unwrap();
        Ok(result)
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

    // TODO: Maybe make it random to be in line with rpc spec?
    fn rpc_id(&self) -> String {
        "1".to_string()
    }

    fn rpc_id_typed(&self) -> JsonRpcId {
        JsonRpcId::String("1".to_string())
    }
}

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
