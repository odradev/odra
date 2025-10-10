//! Client for interacting with Casper node.

use itertools::Itertools;

use crate::casper_client::configuration::CasperClientConfiguration;

use crate::error::LivenetError;
use crate::error::LivenetError::{
    BlockTimeError, ClientError, DictQueryError, ExecutionError, RpcRequestError,
    SerializationError
};
use crate::log;
use crate::utils::extract_stored_value;
use casper_client::cli::{
    get_account, get_dictionary_item, get_node_status, get_state_root_hash,
    DictionaryItemStrParams, TransactionV1Builder
};
use casper_client::rpcs::results::{GetDeployResult, GetTransactionResult};
use casper_client::rpcs::GlobalStateIdentifier;
use casper_client::{
    get_auction_info, get_balance, get_chainspec, get_deploy, get_transaction, put_transaction,
    query_global_state, JsonRpcId
};
use casper_types::bytesrepr::{deserialize_from_slice, Bytes, ToBytes};
use casper_types::execution::ExecutionResultV1::{Failure, Success};
use casper_types::system::auction::{BidAddr, ValidatorBid};
use casper_types::StoredValue::CLValue;
use casper_types::{
    execution::ExecutionResult, runtime_args, sign, CLTyped, Digest, EntityAddr, Key, PricingMode,
    PublicKey, RuntimeArgs, SecretKey, TimeDiff, Transaction, TransactionHash,
    TransactionRuntimeParams, TransferTarget, URef, U512
};
use casper_types::{DeployHash, StoredValue, Timestamp};
use odra_core::casper_event_standard::EVENTS_LENGTH;
use odra_core::consts::{
    AMOUNT_ARG, ARGS_ARG, ATTACHED_VALUE_ARG, CONTRACT_MAIN_PURSE, ENTRY_POINT_ARG, EVENTS,
    PACKAGE_HASH_ARG, PACKAGE_HASH_KEY_NAME_ARG, RESULT_KEY, STATE_KEY
};
use odra_core::prelude::*;
use odra_core::CallDef;
use rand::random;
use toml::Value;

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
/// Environment variable holding TTL for transactions.
pub const ENV_TTL: &str = "ODRA_CASPER_LIVENET_TTL";
/// Time between retries when waiting for a deployment to be processed.
pub const TRANSACTION_WAIT_TIME: u64 = 10;
/// Maximum number of retries when waiting for a transaction to be processed.
pub const TRANSACTION_MAX_RETRIES: u64 = 12;

pub type Result<T> = core::result::Result<T, LivenetError>;

/// Client for interacting with Casper node.
pub struct CasperClient {
    pub configuration: CasperClientConfiguration,
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
            .query_global_state_maybe(Key::Hash(entity_hash.value()), Some(name.to_string()))
            .await;
        match stored_value {
            None => None,
            Some(value) => match value {
                CLValue(value) => Some(Bytes::from(value.inner_bytes().as_slice())),
                _ => {
                    panic!(
                        "Couldn't get {} from {:?}, instead of CLValue got {:?}",
                        name,
                        address.to_formatted_string(),
                        value
                    )
                }
            }
        }
    }

    /// Gets a value from a result key
    pub async fn get_proxy_result(&self) -> Bytes {
        let stored_value = self
            .query_global_state_maybe(self.caller().as_key(), Some(RESULT_KEY.to_string()))
            .await;

        match stored_value {
            None => {
                panic!(
                    "Couldn't query {} from {:?}, instead of CLValue got None",
                    RESULT_KEY,
                    self.caller().to_formatted_string()
                )
            }
            Some(sv) => extract_stored_value(sv)
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

    pub async fn get_validator(&self, index: usize) -> PublicKey {
        let auction_info = get_auction_info(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            None
        )
        .await;
        let auction_info = auction_info.unwrap_or_else(|e| {
            panic!(
                "Couldn't get auction info from node: {:?}, reason: {:?}",
                self.configuration.node_address(),
                e
            )
        });
        let auction_info = auction_info.result;
        let validator = auction_info
            .auction_state
            .era_validators()
            .nth(0)
            .unwrap_or_else(|| panic!("Couldn't get auction state",))
            .validator_weights()
            .nth(index)
            .unwrap()
            .public_key();
        validator.clone()
    }

    pub async fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        let purse_uref = self.get_main_purse(&delegator).await;
        let account_hash = validator.to_account_hash();
        let key = Key::BidAddr(BidAddr::DelegatedPurse {
            validator: account_hash,
            delegator: purse_uref.addr()
        });

        let stored_value = self.query_global_state_maybe(key, None).await;
        match stored_value {
            None => U512::zero(),
            Some(sv) => match sv {
                StoredValue::BidKind(bid_kind) => bid_kind.staked_amount().unwrap_or_default(),
                _ => {
                    panic!(
                        "Couldn't get delegated amount for address: {:?}",
                        delegator.to_formatted_string()
                    )
                }
            }
        }
    }

    pub async fn get_validator_info(&self, _validator: PublicKey) -> Option<ValidatorBid> {
        todo!("Implement get_validator_info")
    }

    pub async fn auction_delay(&self) -> u64 {
        let chainspec = self.chainspec().await;

        let auction_delay = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get auction from chainspec"))
            .get("auction_delay")
            .unwrap_or_else(|| {
                panic!("Couldn't get era_delay from chainspec");
            });

        let era_duration = Self::era_duration(&chainspec);

        era_duration * auction_delay.as_integer().unwrap() as u64
    }

    pub async fn unbonding_delay(&self) -> u64 {
        let chainspec = self.chainspec().await;

        let unbonding_delay = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get core from chainspec"))
            .get("unbonding_delay")
            .unwrap_or_else(|| {
                panic!("Couldn't get unbonding_delay from chainspec");
            });

        let era_duration = Self::era_duration(&chainspec);

        unbonding_delay.as_integer().unwrap() as u64 * era_duration
    }

    fn era_duration(chainspec: &Value) -> u64 {
        let era_duration = chainspec
            .get("core")
            .unwrap_or_else(|| panic!("Couldn't get core from chainspec"))
            .get("era_duration")
            .unwrap_or_else(|| {
                panic!("Couldn't get era_duration from chainspec");
            });
        TimeDiff::from_str(era_duration.as_str().unwrap())
            .unwrap_or_else(|_| {
                panic!(
                    "Couldn't parse era_duration from chainspec: {:?}",
                    era_duration
                )
            })
            .millis()
    }

    async fn chainspec(&self) -> Value {
        let chainspec = get_chainspec(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed()
        )
        .await
        .unwrap_or_else(|e| {
            panic!(
                "Couldn't get chainspec from node: {:?}, reason: {:?}",
                self.configuration.node_address(),
                e
            )
        })
        .result;
        let toml_bytes: &[u8] = chainspec.chainspec_bytes.chainspec_bytes();
        let toml = String::from_utf8_lossy(toml_bytes);
        toml.parse::<Value>()
            .unwrap_or_else(|_| panic!("Couldn't parse chainspec bytes: {:?}", toml))
    }

    /// Sets the amount of gas for the next deployment.
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas.into();
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
            .map_err(|_| SerializationError)?;

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
            self.configuration.node_address(),
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

    /// Gets an uref for a main purse of an account or a contract.
    pub async fn get_main_purse(&self, address: &Address) -> URef {
        let maybe_purse_uref = self.query_global_state_maybe(address.as_key(), None).await;
        let purse_uref_value = match maybe_purse_uref {
            None => {
                panic!(
                    "Couldn't get purse uref for address: {:?}",
                    address.to_formatted_string()
                )
            }
            Some(p) => p
        };

        match purse_uref_value {
            CLValue(value) => value.into_t().unwrap(),
            StoredValue::AddressableEntity(entity) => entity.main_purse(),
            StoredValue::Account(account) => account.main_purse(),
            StoredValue::ContractPackage(contract_package) => {
                let last_version = contract_package.current_contract_hash().unwrap();
                let maybe_contract = self
                    .query_global_state_maybe(Key::Hash(last_version.value()), None)
                    .await;
                let contract_value = match maybe_contract {
                    None => {
                        panic!(
                            "Couldn't get contract for address: {:?}",
                            address.to_formatted_string()
                        )
                    }
                    Some(c) => c
                };
                match contract_value {
                    StoredValue::Contract(contract) => contract
                        .named_keys()
                        .get(CONTRACT_MAIN_PURSE)
                        .unwrap()
                        .into_uref()
                        .unwrap(),
                    _ => panic!(
                        "Couldn't get main purse for address: {:?}",
                        address.to_formatted_string()
                    )
                }
            }
            _ => panic!(
                "Getting main purse is not supported for: {:?}",
                purse_uref_value
            )
        }
    }

    pub async fn transfer(&self, to: Address, amount: U512, timestamp: Timestamp) -> Result<()> {
        let transaction = self.new_transfer_transaction(to, amount, timestamp);
        self.put_transaction(transaction).await?;
        Ok(())
    }

    /// Returns the current block_time
    pub async fn get_block_time(&self) -> Result<u64> {
        let block_time = get_node_status(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity()
        )
        .await
        .map_err(|_| BlockTimeError)?
        .result
        .last_added_block_info
        .ok_or(BlockTimeError)?
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
            self.configuration.node_address(),
            self.configuration.verbosity(),
            ""
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Couldn't get state root hash from node: {:?}",
                self.configuration.node_address()
            )
        })
        .result
        .state_root_hash
        .unwrap_or_else(|| {
            panic!(
                "Couldn't get state root hash from node: {:?}",
                self.configuration.node_address()
            )
        })
    }

    /// Query the node for the dictionary item of a contract or an account.
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

        let r = get_dictionary_item(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity(),
            &self.get_state_root_hash().await,
            params
        )
        .await;

        let result = r.map_err(|e| ClientError(e.to_string()))?;
        let stored_value = result.result.stored_value;
        let cl_value = stored_value.into_cl_value().ok_or(DictQueryError)?;

        // Note: this is for compatibility with CEP18 named keys.
        if cl_value.cl_type() == &Vec::<u8>::cl_type() {
            let bytes = cl_value.into_t().map_err(|_| DictQueryError)?;
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
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
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
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
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
        let result = get_account(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity(),
            "",
            &self.public_key().to_hex_string()
        )
        .await
        .unwrap_or_else(|e| {
            panic!(
                "{}",
                format!("Couldn't get entity for key: {:?}, reason: {}", key_name, e)
            );
        })
        .result;
        let account = result.account;

        let key = account.named_keys().get(key_name).unwrap_or_else(|| {
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
    async fn query_global_state_for_entity_addr(&self, address: &Address) -> EntityAddr {
        let maybe_result = self.query_global_state_maybe(address.as_key(), None).await;
        let entity_addr_value = match maybe_result {
            None => panic!("Couldn't query for entity address value at {:?}", address),
            Some(entity_addr_value) => entity_addr_value
        };
        match entity_addr_value {
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
                    "Entity addr for {:?} was incorrect: {:?}",
                    address.to_formatted_string(),
                    entity_addr_value
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

        let package_hash_key_name: String = args
            .get(PACKAGE_HASH_KEY_NAME_ARG)
            .unwrap()
            .clone()
            .into_t()
            .unwrap();

        let transaction =
            self.new_wasm_deploy_transaction(Bytes::from(wasm_bytes), args, timestamp);
        self.put_transaction(transaction).await?;

        let address = self.get_contract_address(&package_hash_key_name).await;
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

        let module_bytes = include_bytes!("../resources/proxy_caller_with_return.wasm")
            .to_vec()
            .into();

        let transaction = self.new_wasm_deploy_transaction(module_bytes, args, timestamp);
        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await;
        let deploy_hash = response.unwrap().result.transaction_hash;
        let result = self.wait_for_transaction(deploy_hash).await?;
        self.process_transaction(result, deploy_hash)?;
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

        let transaction = self.new_call_transaction(addr, call_def, timestamp);

        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await;
        let deploy_hash = match response {
            Ok(r) => r.result.transaction_hash,
            Err(e) => {
                return match e {
                    casper_client::Error::ResponseIsRpcError {
                        rpc_method, error, ..
                    } => Err(RpcRequestError(
                        rpc_method.to_string(),
                        error
                            .data
                            .map_or_else(|| "No data".to_string(), |d| d.to_string())
                    )),
                    _ => Err(ExecutionError(e.to_string()))
                }
            }
        };
        let result = self.wait_for_transaction(deploy_hash).await?;
        self.process_transaction(result, deploy_hash).map(|_| {
            ().to_bytes()
                .expect("Couldn't serialize (). This shouldn't happen.")
                .into()
        })
    }

    async fn query_global_state_maybe(
        &self,
        key: Key,
        path: Option<String>
    ) -> Option<StoredValue> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let result = query_global_state(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            GlobalStateIdentifier::StateRootHash(self.get_state_root_hash_digest().await),
            key,
            path
        )
        .await;
        match result {
            Ok(r) => Some(r.result.stored_value),
            Err(_) => None
        }
    }

    async fn wait_for_transaction(
        &self,
        transaction_hash: TransactionHash
    ) -> Result<ExecutionResult> {
        let mut retries = TRANSACTION_MAX_RETRIES;

        loop {
            if retries == 0 {
                return Err(ExecutionError(String::from(
                    "RPC is unable to process transaction in time."
                )));
            }

            log::wait(format!(
                "Waiting {:?} for {:?}.",
                &TRANSACTION_WAIT_TIME, &transaction_hash
            ));

            tokio::time::sleep(std::time::Duration::from_secs(TRANSACTION_WAIT_TIME)).await;

            let transaction_info = self.get_transaction(transaction_hash).await;

            if let Some(deploy_info) = transaction_info.execution_info {
                if let Some(execution_result) = deploy_info.execution_result {
                    return Ok(execution_result);
                }
            }

            retries -= 1;
        }
    }

    async fn put_transaction(&self, transaction: Transaction) -> Result<()> {
        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await;
        let deploy_hash = response.unwrap().result.transaction_hash;
        let result = self.wait_for_transaction(deploy_hash).await?;
        self.process_transaction(result, deploy_hash)?;
        Ok(())
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
                    Err(ExecutionError(error_message.to_string()))
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
                        "Transaction {:?} successfully executed.",
                        &deploy_hash_str,
                    ));
                    log::link(self.configuration.transaction_url(&deploy_hash_str));
                    Ok(())
                }
                Some(error_message) => {
                    log::error(format!(
                        "Transaction {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message,
                    ));
                    log::link(self.configuration.transaction_url(&deploy_hash_str));
                    Err(ExecutionError(error_message.to_string()))
                }
            }
        }
    }

    fn new_wasm_deploy_transaction(
        &self,
        transaction_bytes: Bytes,
        args: RuntimeArgs,
        timestamp: Timestamp
    ) -> Transaction {
        let transaction_builder = TransactionV1Builder::new_session(
            true,
            transaction_bytes,
            TransactionRuntimeParams::VmCasperV1
        );
        Transaction::V1(
            transaction_builder
                .with_runtime_args(args)
                .with_ttl(self.configuration.ttl())
                .with_chain_name(self.configuration.chain_name())
                .with_pricing_mode(self.pricing_mode())
                .with_secret_key(self.secret_key())
                .with_timestamp(timestamp)
                .build()
                .unwrap_or_else(|e| panic!("Failed to build transaction: {:?}", e))
        )
    }

    fn new_transfer_transaction(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> Transaction {
        let transaction_builder = TransactionV1Builder::new_transfer(amount, None, TransferTarget::AccountHash(*to.as_account_hash().unwrap_or_else(
            || panic!("Couldn't get account hash from address: {:?}. You can transfer only to accounts.", to)
        )) , None).unwrap_or_else(
            |e| panic!("Failed to build transfer transaction: {:?}", e)
        );
        Transaction::V1(
            transaction_builder
                .with_ttl(self.configuration.ttl())
                .with_chain_name(self.configuration.chain_name())
                .with_pricing_mode(PricingMode::PaymentLimited {
                    payment_amount: amount.as_u64(),
                    gas_price_tolerance: 5,
                    standard_payment: true
                })
                .with_secret_key(self.secret_key())
                .with_timestamp(timestamp)
                .build()
                .unwrap_or_else(|e| panic!("Failed to build transfer transaction: {:?}", e))
        )
    }

    fn new_call_transaction(
        &self,
        to: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Transaction {
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            to.as_package_hash().unwrap(),
            None,
            call_def.entry_point(),
            TransactionRuntimeParams::VmCasperV1
        );
        Transaction::V1(
            transaction_builder
                .with_ttl(self.configuration.ttl())
                .with_chain_name(self.configuration.chain_name())
                .with_pricing_mode(PricingMode::PaymentLimited {
                    payment_amount: call_def.amount().as_u64() + self.gas.as_u64(),
                    gas_price_tolerance: 5,
                    standard_payment: true
                })
                .with_secret_key(self.secret_key())
                .with_timestamp(timestamp)
                .with_runtime_args(call_def.args().clone())
                .build()
                .unwrap_or_else(|e| panic!("Failed to build call transaction: {:?}", e))
        )
    }

    fn pricing_mode(&self) -> PricingMode {
        PricingMode::PaymentLimited {
            payment_amount: self.gas.as_u64(),
            gas_price_tolerance: self.configuration.gas_price_tolerance(),
            standard_payment: true
        }
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

    fn rpc_id(&self) -> String {
        let random_number: u32 = random();
        random_number.to_string()
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
