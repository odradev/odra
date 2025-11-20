//! Query methods for interacting with Casper node state.

use crate::casper_client::Result;
use crate::error::LivenetError::{ClientError, DictQueryError};
use crate::log;
use crate::utils::extract_stored_value;
use casper_client::cli::{get_account, get_dictionary_item, DictionaryItemStrParams};
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
                    log::error(format!(
                        "Couldn't get {} from {:?}, instead of CLValue got {:?}",
                        name,
                        address.to_formatted_string(),
                        value
                    ));
                    None
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
                log::error(format!(
                    "Couldn't query {} from {:?}, instead of CLValue got None",
                    RESULT_KEY,
                    self.caller().to_formatted_string()
                ));
                Bytes::new()
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
            .map_err(|_| {
                log::error(format!("Couldn't convert key to string: {:?}", key));
            })
            .ok()?;
        self.query_dict(address, dictionary_name.to_string(), key)
            .await
            .ok()
    }

    /// Returns the balance of the account.
    pub async fn get_balance(&self, address: &Address) -> Result<U512> {
        let main_purse = self.get_main_purse(address).await?;
        let response = get_balance(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            self.get_state_root_hash_digest().await?,
            main_purse
        )
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
        let maybe_purse_uref = self.query_global_state_maybe(address.as_key(), None).await;
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
                    .await;
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
        let result = get_account(
            &self.rpc_id(),
            self.configuration.node_address(),
            self.configuration.verbosity(),
            "",
            &self.public_key().to_hex_string()
        )
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
                let last_version = package.current_contract_hash().unwrap_or_else(|| {
                    panic!(
                        "Contract package has no current contract hash for address: {:?}",
                        address.to_formatted_string()
                    )
                });
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
            &self.get_state_root_hash().await?,
            params
        )
        .await;

        let result = r.map_err(|e| ClientError(e.to_string()))?;
        let stored_value = result.result.stored_value;
        let cl_value = stored_value.into_cl_value().ok_or(DictQueryError)?;

        // Note: this is for compatibility with CEP18 named keys.
        if cl_value.cl_type() == &<Vec<u8> as CLTyped>::cl_type() {
            let bytes = cl_value.into_t().map_err(|_| DictQueryError)?;
            Ok(bytes)
        } else {
            let bytes = cl_value.inner_bytes();
            Ok(Bytes::from(bytes.to_vec()))
        }
    }

    pub(crate) async fn query_global_state_maybe(
        &self,
        key: Key,
        path: Option<String>
    ) -> Option<StoredValue> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let state_root_hash = match self.get_state_root_hash_digest().await {
            Ok(hash) => hash,
            Err(_) => return None
        };
        let result = query_global_state(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            GlobalStateIdentifier::StateRootHash(state_root_hash),
            key,
            path
        )
        .await;
        match result {
            Ok(r) => Some(r.result.stored_value),
            Err(_) => None
        }
    }
}
