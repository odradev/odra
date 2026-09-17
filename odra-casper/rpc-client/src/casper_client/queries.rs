//! Query methods for interacting with Casper node state.

use crate::casper_client::Result;
use crate::error::LivenetError::{ClientError, DictQueryError};
use crate::log;
use crate::utils::block_on;
use crate::utils::{extract_stored_value, retry_on_rate_limit, RateLimited};
use casper_client::cli::{get_account, get_dictionary_item, CliError, DictionaryItemStrParams};
use casper_client::rpcs::results::{GetDeployResult, GetTransactionResult};
use casper_client::rpcs::GlobalStateIdentifier;
use casper_client::{get_balance, get_deploy, get_transaction, query_global_state};
use casper_types::bytesrepr::{deserialize_from_slice, Bytes};
use casper_types::StoredValue::CLValue;
use casper_types::{
    CLTyped, DeployHash, EntityAddr, Key, StoredValue, TransactionHash, URef, U512
};
use odra_core::casper_event_standard::EVENTS_LENGTH;
use odra_core::consts::{CONTRACT_MAIN_PURSE, EVENTS, RESULT_KEY, STATE_KEY};
use odra_core::prelude::*;

/// Query methods implementation for CasperClient.
impl super::CasperClient {
    /// Gets a value from the Odra storage (`state` dictionary).
    ///
    /// `Ok(None)` means the node has no such value; `Err` means the node could not be asked.
    pub fn get_value(&self, address: &Address, key: &[u8]) -> Result<Option<Bytes>> {
        block_on(self.get_value_async(address, key))
    }

    /// Gets a value from the Odra storage (`state` dictionary)
    pub async fn get_value_async(&self, address: &Address, key: &[u8]) -> Result<Option<Bytes>> {
        self.get_dictionary_value_async(address, STATE_KEY, key)
            .await
    }

    /// Gets a value from a named key of an account or a contract.
    ///
    /// `Ok(None)` means the node has no such value; `Err` means the node could not be asked.
    pub fn get_named_value(&self, address: &Address, name: &str) -> Result<Option<Bytes>> {
        block_on(self.get_named_value_async(address, name))
    }

    /// Gets a value from a named key of an account or a contract
    pub async fn get_named_value_async(
        &self,
        address: &Address,
        name: &str
    ) -> Result<Option<Bytes>> {
        let entity_hash = self.entity_addr(address).await?;
        let stored_value = self
            .query_global_state_maybe(Key::Hash(entity_hash.value()), Some(name.to_string()))
            .await?;
        match stored_value {
            None => Ok(None),
            Some(CLValue(value)) => Ok(Some(Bytes::from(value.inner_bytes().as_slice()))),
            Some(value) => Err(ClientError(format!(
                "Couldn't get {} from {:?}, instead of CLValue got {:?}",
                name,
                address.to_formatted_string(),
                value
            )))
        }
    }

    /// Gets a value from a result key
    pub async fn get_proxy_result(&self) -> Bytes {
        let stored_value = self
            .query_global_state_maybe(self.caller().as_key(), Some(RESULT_KEY.to_string()))
            .await;

        match stored_value {
            Ok(Some(sv)) => extract_stored_value(sv),
            Ok(None) => {
                log::error(format!(
                    "Couldn't query {} from {:?}, instead of CLValue got None",
                    RESULT_KEY,
                    self.caller().to_formatted_string()
                ));
                Bytes::new()
            }
            Err(e) => {
                log::error(format!(
                    "Couldn't query {} from {:?}: {}",
                    RESULT_KEY,
                    self.caller().to_formatted_string(),
                    e.error_message()
                ));
                Bytes::new()
            }
        }
    }

    /// Gets a value from a named dictionary.
    ///
    /// `Ok(None)` means the node has no such value; `Err` means the node could not be asked.
    pub fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Result<Option<Bytes>> {
        block_on(self.get_dictionary_value_async(address, dictionary_name, key))
    }

    /// Gets a value from a named dictionary
    pub async fn get_dictionary_value_async(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Result<Option<Bytes>> {
        let key = String::from_utf8(key.to_vec())
            .map_err(|_| ClientError(format!("Couldn't convert key to string: {:?}", key)))?;
        self.query_dict(address, dictionary_name.to_string(), key)
            .await
    }

    /// Returns the balance of the account.
    pub fn get_balance(&self, address: &Address) -> Result<U512> {
        block_on(self.get_balance_async(address))
    }

    /// Returns the balance of the account.
    pub async fn get_balance_async(&self, address: &Address) -> Result<U512> {
        let main_purse = self.get_main_purse(address).await?;
        let state_root_hash = self.get_state_root_hash_digest().await?;
        let response = retry_on_rate_limit("state_get_balance", || {
            get_balance(
                self.rpc_id_typed(),
                self.configuration.node_address(),
                self.configuration.verbosity_typed(),
                state_root_hash,
                main_purse
            )
        })
        .await
        .map_err(|e| {
            ClientError(format!(
                "Couldn't get balance for address: {:?}, error: {}",
                address.to_formatted_string(),
                e
            ))
        })?;
        Ok(response.result.balance_value)
    }

    /// Gets an uref for a main purse of an account or a contract.
    pub async fn get_main_purse(&self, address: &Address) -> Result<URef> {
        let maybe_purse_uref = self
            .query_global_state_maybe(address.as_key(), None)
            .await?;
        let purse_uref_value = maybe_purse_uref.ok_or_else(|| {
            ClientError(format!(
                "Couldn't get purse uref for address: {:?}",
                address.to_formatted_string()
            ))
        })?;

        match purse_uref_value {
            CLValue(value) => value.into_t().map_err(|e| {
                ClientError(format!(
                    "Failed to convert CLValue to URef for address: {:?}, error: {:?}",
                    address.to_formatted_string(),
                    e
                ))
            }),
            StoredValue::AddressableEntity(entity) => Ok(entity.main_purse()),
            StoredValue::Account(account) => Ok(account.main_purse()),
            StoredValue::ContractPackage(contract_package) => {
                let last_version = contract_package.current_contract_hash().ok_or_else(|| {
                    ClientError(format!(
                        "Contract package has no current contract hash for address: {:?}",
                        address.to_formatted_string()
                    ))
                })?;
                let maybe_contract = self
                    .query_global_state_maybe(Key::Hash(last_version.value()), None)
                    .await?;
                let contract_value = maybe_contract.ok_or_else(|| {
                    ClientError(format!(
                        "Couldn't get contract for address: {:?}",
                        address.to_formatted_string()
                    ))
                })?;
                match contract_value {
                    StoredValue::Contract(contract) => {
                        let purse_key =
                            contract
                                .named_keys()
                                .get(CONTRACT_MAIN_PURSE)
                                .ok_or_else(|| {
                                    ClientError(format!(
                                        "Contract missing {} named key for address: {:?}",
                                        CONTRACT_MAIN_PURSE,
                                        address.to_formatted_string()
                                    ))
                                })?;
                        purse_key.into_uref().ok_or_else(|| {
                            ClientError(format!(
                                "{} named key is not a URef for address: {:?}",
                                CONTRACT_MAIN_PURSE,
                                address.to_formatted_string()
                            ))
                        })
                    }
                    _ => Err(ClientError(format!(
                        "Couldn't get main purse for address: {:?}",
                        address.to_formatted_string()
                    )))
                }
            }
            _ => Err(ClientError(format!(
                "Getting main purse is not supported for: {:?}",
                purse_uref_value
            )))
        }
    }

    /// Get the event bytes from storage
    pub fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes> {
        block_on(self.get_event_async(contract_address, index))
    }

    /// Get the event bytes from storage
    pub async fn get_event_async(&self, contract_address: &Address, index: u32) -> Result<Bytes> {
        self.query_dict(contract_address, EVENTS.to_string(), index.to_string())
            .await?
            .ok_or_else(|| {
                ClientError(format!(
                    "No event at index {index} for contract {}",
                    contract_address.to_formatted_string()
                ))
            })
    }

    /// Get the events count from storage.
    ///
    /// `Ok(None)` when the contract has no events dictionary.
    pub fn events_count(&self, contract_address: &Address) -> Result<Option<u32>> {
        block_on(self.events_count_async(contract_address))
    }

    /// Get the events count from storage
    pub async fn events_count_async(&self, contract_address: &Address) -> Result<Option<u32>> {
        let bytes = self
            .get_named_value_async(contract_address, EVENTS_LENGTH)
            .await?;
        bytes
            .map(|bytes| {
                deserialize_from_slice(&bytes).map_err(|_| {
                    ClientError(format!(
                        "Couldn't deserialize events count for contract: {:?}, bytes: {:?}",
                        contract_address, bytes
                    ))
                })
            })
            .transpose()
    }

    /// Query the node for the transaction state.
    pub async fn get_transaction(
        &self,
        transaction_hash: TransactionHash
    ) -> Result<GetTransactionResult> {
        let t = get_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction_hash,
            true
        )
        .await
        .map_err(|e| {
            log::error(format!("Couldn't get transaction: {:?}", e));
            ClientError(format!(
                "Couldn't get transaction: {}",
                transaction_hash.to_hex_string()
            ))
        })?;
        Ok(t.result)
    }

    /// Query the node for the transaction state.
    pub async fn get_deploy(&self, deploy_hash: DeployHash) -> Result<GetDeployResult> {
        let t = get_deploy(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            deploy_hash,
            true
        )
        .await
        .map_err(|e| {
            log::error(format!("Couldn't get deploy: {:?}", e));
            ClientError(format!(
                "Couldn't get deploy: {}",
                deploy_hash.to_hex_string()
            ))
        })?;
        Ok(t.result)
    }

    /// Discover the contract address by name.
    pub(crate) async fn get_contract_address(&self, key_name: &str) -> Result<Address> {
        let rpc_id = self.rpc_id();
        let public_key = self.public_key().to_hex_string();
        let result = retry_on_rate_limit("state_get_account_info", || {
            get_account(
                &rpc_id,
                self.configuration.node_address(),
                self.configuration.verbosity(),
                "",
                &public_key
            )
        })
        .await
        .map_err(|e| {
            ClientError(format!(
                "Couldn't get entity for key: {:?}, reason: {}",
                key_name, e
            ))
        })?;
        let account = result.result.account;

        let key = account.named_keys().get(key_name).ok_or_else(|| {
            ClientError(format!(
                "Couldn't get named key {:?} for account: {:?}",
                key_name,
                self.public_key().to_hex_string()
            ))
        })?;

        let package_hash = key.into_package_hash().ok_or_else(|| {
            ClientError(format!(
                "Couldn't get package hash from key {:?} for account: {:?}",
                key_name,
                self.public_key().to_hex_string()
            ))
        })?;

        Ok(Address::from(package_hash))
    }

    /// Resolves the current entity of the contract package at `address`.
    ///
    /// Served from the query cache within one state root hash, so a burst of reads from one
    /// contract resolves it once.
    async fn entity_addr(&self, address: &Address) -> Result<EntityAddr> {
        let entity_addr_value = self
            .query_global_state_maybe(address.as_key(), None)
            .await?
            .ok_or_else(|| {
                ClientError(format!(
                    "No contract found at {}",
                    address.to_formatted_string()
                ))
            })?;
        match entity_addr_value {
            StoredValue::SmartContract(package) => package
                .current_entity_hash()
                .map(|hash| EntityAddr::SmartContract(hash.value()))
                .ok_or_else(|| {
                    ClientError(format!(
                        "Contract package {} has no current entity",
                        address.to_formatted_string()
                    ))
                }),
            StoredValue::ContractPackage(package) => package
                .current_contract_hash()
                .map(|hash| EntityAddr::SmartContract(hash.value()))
                .ok_or_else(|| {
                    ClientError(format!(
                        "Contract package {} has no current contract hash",
                        address.to_formatted_string()
                    ))
                }),
            other => Err(ClientError(format!(
                "Expected a contract package at {}, found {:?}",
                address.to_formatted_string(),
                other
            )))
        }
    }

    /// Query the node for the dictionary item of a contract or an account.
    async fn query_dict(
        &self,
        address: &Address,
        dictionary_name: String,
        dictionary_item_key: String
    ) -> Result<Option<Bytes>> {
        let entity_addr = self.entity_addr(address).await?;
        let hash_addr = Key::Hash(entity_addr.value()).to_formatted_string();
        let state_root_hash_digest = self.get_state_root_hash_digest().await?;
        let cache_key = (
            hash_addr.clone(),
            dictionary_name.clone(),
            dictionary_item_key.clone()
        );
        if let Some(cached) = self.cached_dictionary_item(state_root_hash_digest, &cache_key) {
            return Ok(cached);
        }
        let state_root_hash = base16::encode_lower(&state_root_hash_digest);
        let rpc_id = self.rpc_id();
        let r = retry_on_rate_limit("state_get_dictionary_item", || {
            get_dictionary_item(
                &rpc_id,
                self.configuration.node_address(),
                self.configuration.verbosity(),
                &state_root_hash,
                DictionaryItemStrParams::ContractNamedKey {
                    hash_addr: &hash_addr,
                    dictionary_name: &dictionary_name,
                    dictionary_item_key: &dictionary_item_key
                }
            )
        })
        .await;

        let result = match r {
            Ok(result) => result,
            // The node answered and has no such item: a legitimate miss.
            Err(CliError::Core(e @ casper_client::Error::ResponseIsRpcError { .. }))
                if !e.is_rate_limited() =>
            {
                log::debug(format!(
                    "state_get_dictionary_item({dictionary_name}, {dictionary_item_key}): {e}"
                ));
                self.cache_dictionary_item(state_root_hash_digest, cache_key, None);
                return Ok(None);
            }
            Err(e) => return Err(ClientError(e.to_string()))
        };
        let stored_value = result.result.stored_value;
        let cl_value = stored_value.into_cl_value().ok_or(DictQueryError)?;

        // Note: this is for compatibility with CEP18 named keys.
        let bytes = if cl_value.cl_type() == &<Vec<u8> as CLTyped>::cl_type() {
            cl_value.into_t().map_err(|_| DictQueryError)?
        } else {
            Bytes::from(cl_value.inner_bytes().to_vec())
        };
        self.cache_dictionary_item(state_root_hash_digest, cache_key, Some(bytes.clone()));
        Ok(Some(bytes))
    }

    /// Queries the global state at the current state root hash.
    ///
    /// `Ok(None)` means the node answered that there is no such value; `Err` means the node could
    /// not be asked (transport failure, or still throttled after retries).
    pub(crate) async fn query_global_state_maybe(
        &self,
        key: Key,
        path: Option<String>
    ) -> Result<Option<StoredValue>> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let state_root_hash = self.get_state_root_hash_digest().await?;
        if let Some(cached) = self.cached_global_state(state_root_hash, &key, &path) {
            return Ok(cached);
        }
        let result = retry_on_rate_limit("query_global_state", || {
            query_global_state(
                self.rpc_id_typed(),
                self.configuration.node_address(),
                self.configuration.verbosity_typed(),
                GlobalStateIdentifier::StateRootHash(state_root_hash),
                key,
                path.clone()
            )
        })
        .await;
        match result {
            Ok(r) => {
                let value = r.result.stored_value;
                self.cache_global_state(state_root_hash, key, path, Some(value.clone()));
                Ok(Some(value))
            }
            // The node answered; a missing value is a legitimate `None` for optional lookups.
            Err(e @ casper_client::Error::ResponseIsRpcError { .. }) if !e.is_rate_limited() => {
                log::debug(format!("query_global_state({key:?}): {e}"));
                self.cache_global_state(state_root_hash, key, path, None);
                Ok(None)
            }
            // Throttled even after retries, or a transport/HTTP failure.
            Err(e) => Err(ClientError(format!(
                "query_global_state({key:?}) failed: {e}"
            )))
        }
    }
}
