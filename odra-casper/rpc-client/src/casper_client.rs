//! Client for interacting with Casper node.
pub mod configuration;

use std::{fs, path::PathBuf, str::from_utf8_unchecked, time::Duration};
use casper_client::cli::{get_state_root_hash, query_balance, query_global_state};
use casper_client::rpcs::results::{PutDeployResult, QueryGlobalStateResult};

use itertools::Itertools;
use jsonrpc_lite::JsonRpc;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tokio::runtime::Runtime;

use odra_core::casper_types::{DeployHash, Digest, ExecutableDeployItem, sign, URef};
use odra_core::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        runtime_args, CLTyped, Key as CasperKey, PublicKey, RuntimeArgs, SecretKey, TimeDiff,
        Timestamp, U512
    },
    consts::*,
    Address, CallDef, ExecutionError, OdraError, OdraResult
};
use odra_core::casper_types::contracts::ContractHash;
use odra_core::casper_types::execution::ExecutionResult;
use odra_core::casper_types::StoredValue::{Account, CLValue};
use crate::casper_client::configuration::CasperClientConfiguration;

use crate::log;
use crate::utils::find_wasm_file_path;

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

enum ContractId {
    Name(String),
    Address(Address)
}

fn get_env_variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|err| {
        log::error(format!(
            "{} must be set. Have you setup your .env file?",
            name
        ));
        panic!("{}", err)
    })
}

fn get_optional_env_variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

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

    /// Gets a value from the storage
    pub fn get_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.query_state_dictionary(address, unsafe { from_utf8_unchecked(key) })
    }

    /// Gets a value from a named key
    pub fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        let uref = self.query_contract_named_key(address, name).unwrap();
        self.query_uref_bytes(uref).ok()
    }

    /// Gets a value from a named dictionary
    pub fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        let key = String::from_utf8(key.to_vec()).unwrap();
        self.query_dict_bytes(address, dictionary_name.to_string(), key)
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

    /// Secret keys of the client.
    pub fn secret_keys(&self) -> &Vec<SecretKey> {
        &self.configuration.secret_keys
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
    pub fn get_balance(&self, address: &Address) -> U512 {
        let rt = Runtime::new().unwrap();
        let address_string = address.to_string();
        let balance = rt
            .block_on(async {
                let state_root_hash = get_state_root_hash(
                    "",
                    self.configuration.node_address.as_str(),
                    2,
                    "",
                ).await;
                let state_root_hash = state_root_hash.unwrap().result.state_root_hash.unwrap().value();
                let state_root_hash = hex::encode(state_root_hash);
                let global_state = query_global_state(
                    "",
                    self.configuration.node_address.as_str(),
                    2,
                    "",
                    &state_root_hash,
                    "",
                    ""
                );
                dbg!(global_state.await);
                query_balance(
                    "",
                    self.configuration.node_address.as_str(),
                    2,
                    "",
                    &state_root_hash,
                    &address_string
                )
                .await
            })
            .unwrap();
        balance.result.balance
        //     Some(GlobalStateIdentifier::StateRootHash(
        //         self.get_state_root_hash()
        //     )),
        //     PurseIdentifier::MainPurseUnderAccountHash(
        //         *address
        //             .as_account_hash()
        //             .unwrap_or_else(|| panic!("Address {:?} is not an account address", address))
        //     )
        // );
        // let request = json!(
        //     {
        //         "jsonrpc": "2.0",
        //         "method": QUERY_BALANCE_METHOD,
        //         "params": query_balance_params,
        //         "id": 1,
        //     }
        // );
        // let result: QueryBalanceResult = self.post_request(request);
        // result.balance
    }

    /// Returns the current block_time
    pub fn get_block_time(&self) -> u64 {
        let request = json!(
            {
                "jsonrpc": "2.0",
                "method": "info_get_status",
                "id": 1,
            }
        );
        let result: serde_json::Value = self.post_request(request);
        let result = result["result"]["last_added_block_info"]["timestamp"]
            .as_str()
            .unwrap_or_else(|| {
                panic!(
                    "Couldn't get block time - malformed JSON response: {:?}",
                    result
                )
            });
        let timestamp: u64 = result.parse().expect("Couldn't parse block time");
        timestamp
    }

    /// Get the event bytes from storage
    pub fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, OdraError> {
        let event_bytes: Bytes =
            self.query_dict(contract_address, "__events".to_string(), index.to_string())?;
        Ok(event_bytes)
    }

    /// Get the events count from storage
    pub fn events_count(&self, contract_address: &Address) -> u32 {
        let uref = self
            .query_contract_named_key(contract_address, "__events_length")
            .unwrap();
        self.query_uref(uref).unwrap()
    }

    /// Query the node for the current state root hash.
    fn get_state_root_hash(&self) -> Digest {
        let rt = Runtime::new().unwrap();
        rt.block_on(
            async {
                get_state_root_hash(
                    "",
                    self.configuration.node_address.as_str(),
                    2,
                    "",
                )
                .await
            }
        ).unwrap().result.state_root_hash.unwrap()
    }

    /// Query the node for the dictionary item of a contract.
    fn query_dict<T: FromBytes + CLTyped>(
        &self,
        contract_address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<T, OdraError> {
        todo!()
    }

    fn query_dict_bytes(
        &self,
        contract_address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<Bytes, OdraError> {
        todo!();
        // let state_root_hash = self.get_state_root_hash();
        // let contract_hash = self.query_global_state_for_contract_hash(contract_address);
        // let contract_hash = contract_hash
        //     .to_formatted_string()
        //     .replace("contract-", "hash-");
        // let params = GetDictionaryItemParams {
        //     state_root_hash,
        //     dictionary_identifier: DictionaryIdentifier::ContractNamedKey {
        //         key: contract_hash,
        //         dictionary_name,
        //         dictionary_item_key
        //     }
        // };
        //
        // let request = json!(
        //     {
        //         "jsonrpc": "2.0",
        //         "method": "state_get_dictionary_item",
        //         "params": params,
        //         "id": 1,
        //     }
        // );
        // let result: GetDictionaryItemResult = self.post_request(request);
        // match result.stored_value {
        //     CLValue(value) => {
        //         let return_value: Bytes = Bytes::from(value.inner_bytes().as_slice());
        //         Ok(return_value)
        //     }
        //     _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        // }
    }

    /// Query the contract for the direct value of a named key
    pub fn query_contract_named_key<T: AsRef<str>>(
        &self,
        contract_address: &Address,
        key: T
    ) -> Option<URef> {
        todo!()
        // let contract_state =
        //     self.query_global_state_path(contract_address, key.as_ref().to_string());
        // let uref_str = match contract_state.as_ref().unwrap().stored_value.clone() {
        //     Contract(contract) => contract.named_keys().find_map(|named_key| {
        //         if named_key.name == key.as_ref() {
        //             Some(named_key.key.clone())
        //         } else {
        //             None
        //         }
        //     }),
        //     _ => panic!("Not a contract")
        // }
        // .unwrap_or_else(|| {
        //     panic!(
        //         "Couldn't get named key {} from contract state at address {:?}",
        //         key.as_ref(),
        //         contract_address
        //     )
        // });
        // URef::from_formatted_str(&uref_str).ok()
    }

    /// Query the node for the deploy state.
    // pub fn get_deploy(&self, deploy_hash: DeployHash) -> GetDeployResult {
    // let params = GetDeployParams {
    //     deploy_hash,
    //     finalized_approvals: false
    // };
    //
    // let request = json!(
    //     {
    //         "jsonrpc": "2.0",
    //         "method": "info_get_deploy",
    //         "params": params,
    //         "id": 1,
    //     }
    // );
    // self.post_request(request)
    // }

    /// Discover the contract address by name.
    fn get_contract_address(&self, key_name: &str) -> Address {
        todo!()
        // let key_name = format!("{}_package_hash", key_name);
        // let account_hash = self.public_key().to_account_hash();
        // let result = self.query_global_state(&CasperKey::Account(account_hash));
        // let key = match result.stored_value {
        //     Account(account) => account
        //         .named_keys()
        //         .find(|named_key| named_key.name == key_name)
        //         .map_or_else(
        //             || {
        //                 panic!(
        //                     "Couldn't get contract address from account state at key {}",
        //                     key_name
        //                 )
        //             },
        //             |named_key| named_key.key.clone()
        //         ),
        //     _ => panic!(
        //         "Couldn't get contract address from account state at key {}",
        //         key_name
        //     )
        // }
        // .as_str()
        // .replace("hash-", "contract-package-wasm");
        // let contract_hash = ContractPackageHash::from_formatted_str(&key).unwrap();
        // Address::try_from(contract_hash).unwrap()
    }

    /// Find the contract hash by the contract package hash.
    fn query_global_state_for_contract_hash(&self, address: &Address) -> ContractHash {
        todo!()
        // let key = CasperKey::Hash(address.as_contract_package_hash().unwrap().value());
        // let result = self.query_global_state(&key);
        // let result_as_json = serde_json::to_value(result).unwrap();
        // let contract_hash: &str = result_as_json["stored_value"]["ContractPackage"]["versions"][0]
        //     ["contract_hash"]
        //     .as_str()
        //     .unwrap();
        // ContractHash::from_formatted_str(contract_hash).unwrap()
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

        let response: PutDeployResult = self.post_request(request);
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);

        let address = self.get_contract_address(contract_name);
        log::info(format!("Contract {:?} deployed.", &address.to_string()));
        address
    }

    /// Deploy the entrypoint call using getter_proxy.
    /// It runs the getter_proxy contract in an account context and stores the return value of the call
    /// in under the key RESULT_KEY.
    pub fn deploy_entrypoint_call_with_proxy(
        &self,
        addr: Address,
        call_def: CallDef
    ) -> Result<Bytes, OdraError> {
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
        let response: PutDeployResult = self.post_request(request);
        let deploy_hash = response.deploy_hash;
        self.wait_for_deploy_hash(deploy_hash);

        let r = self.query_global_state(&CasperKey::Account(self.public_key().to_account_hash()));
        match r.stored_value {
            Account(account) => {
                todo!()
                // let result = account
                //     .named_keys()
                //     .find(|named_key| named_key.name == RESULT_KEY);
                // match result {
                //     Some(result) => self.query_uref(URef::from_formatted_str(&result.key).unwrap()),
                //     None => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
                // }
            }
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    /// Deploy the entrypoint call.
    pub fn deploy_entrypoint_call(&self, addr: Address, call_def: CallDef) {
        todo!()
        // log::info(format!(
        //     "Calling {:?} with entrypoint \"{}\".",
        //     addr.to_string(),
        //     call_def.entry_point()
        // ));
        // let session = ExecutableDeployItem::StoredVersionedContractByHash {
        //     hash: *addr.as_contract_package_hash().unwrap(),
        //     version: None,
        //     entry_point: call_def.entry_point().to_string(),
        //     args: call_def.args().clone()
        // };
        // let deploy = self.new_deploy(session, self.gas);
        // let request = json!(
        //     {
        //         "jsonrpc": "2.0",
        //         "method": "account_put_deploy",
        //         "params": {
        //             "deploy": deploy
        //         },
        //         "id": 1,
        //     }
        // );
        // let response: PutDeployResult = self.post_request(request);
        // let deploy_hash = response.deploy_hash;
        // self.wait_for_deploy_hash(deploy_hash);
    }

    fn query_global_state_path(
        &self,
        address: &Address,
        _path: String
    ) -> Option<QueryGlobalStateResult> {
        todo!()
        // let hash = self.query_global_state_for_contract_hash(address);
        // let key = CasperKey::Hash(hash.value());
        // let state_root_hash = self.get_state_root_hash();
        // let params = QueryGlobalStateParams {
        //     state_identifier: GlobalStateIdentifier::StateRootHash(state_root_hash),
        //     key: key.to_formatted_string(),
        //     path: vec![]
        // };
        // let request = json!(
        //     {
        //         "jsonrpc": "2.0",
        //         "method": "query_global_state",
        //         "params": params,
        //         "id": 1,
        //     }
        // );
        // self.post_request(request)
    }

    fn query_global_state(&self, key: &CasperKey) -> QueryGlobalStateResult {
        todo!()
    //     let state_root_hash = self.get_state_root_hash();
    //     let params = QueryGlobalStateParams {
    //         state_identifier: GlobalStateIdentifier::StateRootHash(state_root_hash),
    //         key: key.to_formatted_string(),
    //         path: Vec::new()
    //     };
    //     let request = json!(
    //         {
    //             "jsonrpc": "2.0",
    //             "method": "query_global_state",
    //             "params": params,
    //             "id": 1,
    //         }
    //     );
    //     self.post_request(request)
    }

    fn query_state_dictionary(&self, address: &Address, key: &str) -> Option<Bytes> {
        todo!()
        // let state_root_hash = self.get_state_root_hash();
        // let contract_hash = self.query_global_state_for_contract_hash(address);
        // let contract_hash = contract_hash
        //     .to_formatted_string()
        //     .replace("contract-", "hash-");
        // let params = GetDictionaryItemParams {
        //     state_root_hash,
        //     dictionary_identifier: DictionaryIdentifier::ContractNamedKey {
        //         key: contract_hash,
        //         dictionary_name: String::from("state"),
        //         dictionary_item_key: String::from(key)
        //     }
        // };
        //
        // let request = json!(
        //     {
        //         "jsonrpc": "2.0",
        //         "method": "state_get_dictionary_item",
        //         "params": params,
        //         "id": 1,
        //     }
        // );
        // let result: Option<GetDictionaryItemResult> = self.post_request(request);
        // result.map(|result| {
        //     let result_as_json = serde_json::to_value(result).unwrap();
        //     let result = result_as_json["stored_value"]["CLValue"]["bytes"]
        //         .as_str()
        //         .unwrap();
        //     let bytes = hex::decode(result).unwrap();
        //     let (value, _) = FromBytes::from_bytes(&bytes).unwrap();
        //     value
        // })
    }

    fn query_uref<T: CLTyped + FromBytes>(&self, uref: URef) -> OdraResult<T> {
        let result = self.query_global_state(&CasperKey::URef(uref));
        match result.stored_value {
            CLValue(value) => {
                let return_value: T = value.into_t().unwrap();
                Ok(return_value)
            }
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    fn query_uref_bytes(&self, uref: URef) -> OdraResult<Bytes> {
        let result = self.query_global_state(&CasperKey::URef(uref));
        match result.stored_value {
            CLValue(value) => Ok(Bytes::from(value.inner_bytes().as_slice())),
            _ => Err(OdraError::ExecutionError(ExecutionError::TypeMismatch))
        }
    }

    fn wait_for_deploy_hash(&self, deploy_hash: DeployHash) -> ExecutionResult {
        todo!()
    }
    // let deploy_hash_str = format!("{:?}", deploy_hash.inner());
    // let time_diff = Duration::from_secs(15);
    // let final_result;

        // loop {
        //     log::wait(format!(
        //         "Waiting {:?} for {:?}.",
        //         &time_diff, &deploy_hash_str
        //     ));
        //     std::thread::sleep(time_diff);
        //     let result: GetDeployResult = self.get_deploy(deploy_hash);
        //     if !result.execution_results.is_empty() {
        //         final_result = result;
        //         break;
        //     }
        // }
        //
        // match &final_result.execution_results[0].result {
        //     ExecutionResult::Failure {
        //         effect: _,
        //         transfers: _,
        //         cost: _,
        //         error_message
        //     } => {
        //         log::error(format!(
        //             "Deploy {:?} failed with error: {:?}.",
        //             deploy_hash_str, error_message
        //         ));
        //         panic!("Deploy failed");
        //     }
        //     ExecutionResult::Success {
        //         effect: _,
        //         transfers: _,
        //         cost: _
        //     } => {
        //         log::info(format!(
        //             "Deploy {:?} successfully executed.",
        //             deploy_hash_str
        //         ));
        //         final_result.execution_results[0].result.clone()
        //     }
        // }
    // }

    fn new_deploy(&self, session: ExecutableDeployItem, gas: U512) -> DeployHash {
        todo!()
    }
    //     let timestamp = Timestamp::now();
    //     let ttl = TimeDiff::from_seconds(1000);
    //     let gas_price = 1;
    //     let dependencies = vec![];
    //     let chain_name = String::from(self.chain_name());
    //     let payment = ExecutableDeployItem::ModuleBytes {
    //         module_bytes: Default::default(),
    //         args: runtime_args! {
    //             "amount" => gas
    //         }
    //     };

    // Deploy::new(
    //     timestamp,
    //     ttl,
    //     gas_price,
    //     dependencies,
    //     chain_name,
    //     payment,
    //     session,
    //     self.secret_key(),
    //     Some(self.public_key())
    // )
    // }

    fn post_request<T: DeserializeOwned>(&self, request: Value) -> T {
        let client = reqwest::blocking::Client::new();
        let response = client.post(self.node_address_rpc()).json(&request).send();
        let response = match response {
            Ok(r) => r,
            Err(e) => {
                log::error(format!("Couldn't send request: {:?}", e));
                panic!("Couldn't send request")
            }
        };
        let json: JsonRpc = response.json().unwrap_or_else(|e| {
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
            .unwrap_or_else(|| {
                log::error(format!("Couldn't get result: {:?}", json));
                panic!("Couldn't get result")
            })
    }

    fn address_secret_key(&self, address: &Address) -> &SecretKey {
        match self
            .configuration
            .secret_keys
            .iter()
            .find(|key| Address::from(PublicKey::from(*key)) == *address)
        {
            Some(secret_key) => secret_key,
            None => panic!("Key for address {:?} is not loaded", address)
        }
    }
}

impl Default for CasperClient {
    fn default() -> Self {
        Self::new(CasperClientConfiguration::from_env())
    }
}

#[cfg(test)]
mod tests {
    use crate::casper_client::configuration::CasperClientConfiguration;

    #[test]
    fn test_client_setup() {
        let configuration = CasperClientConfiguration {
            node_address: "http://localhost:7777".to_string(),
            chain_name: "casper-test".to_string(),
            secret_keys: vec![]
        };

        let client = super::CasperClient::default();
        let caller = client.get_account(0);

        let balance = client.get_balance(&caller);

        assert_eq!(balance, 0.into());

        // Test your contract here.
    }
}
