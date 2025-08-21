use std::str::FromStr;

use casper_client::cli::TransactionV1Builder;
use casper_types::{
    EntityAddr, Key, PricingMode, RuntimeArgs, StoredValue, TimeDiff, Timestamp, Transaction,
    TransactionHash, TransactionRuntimeParams, URef
};
use js_sys::Date;
use odra_core::prelude::Address;

use crate::OdraWasmClient;

impl OdraWasmClient {
    /// Gets an uref for a main purse of an account or a contract.
    pub async fn get_main_purse(&self, address: &Address) -> Result<URef, String> {
        let purse_uref_value = self
            .query_global_state(address.as_key(), None)
            .await
            .ok_or(format!(
                "Couldn't get purse uref for address: {:?}",
                address.to_formatted_string()
            ))?;

        let result = match purse_uref_value {
            StoredValue::CLValue(value) => value.into_t().map_err(|_| {
                format!(
                    "Couldn't get CLValue for address: {:?}",
                    address.to_formatted_string()
                )
            }),
            StoredValue::AddressableEntity(entity) => Ok(entity.main_purse()),
            StoredValue::Account(account) => Ok(account.main_purse()),
            StoredValue::ContractPackage(contract_package) => {
                let last_version = contract_package.current_contract_hash().ok_or(format!(
                    "Couldn't get last version for address: {:?}",
                    address.to_formatted_string()
                ))?;
                let maybe_contract = self
                    .query_global_state(Key::Hash(last_version.value()), None)
                    .await;
                match maybe_contract {
                    Some(StoredValue::Contract(contract)) => contract
                        .named_keys()
                        .get("__contract_main_purse")
                        .map(|v| v.into_uref())
                        .flatten()
                        .ok_or(format!(
                            "Couldn't get main purse for address: {:?}",
                            address.to_formatted_string()
                        )),
                    _ => Err(format!(
                        "Couldn't get main purse for address: {:?}",
                        address.to_formatted_string()
                    ))
                }
            }
            _ => {
                return Err(format!(
                    "Getting main purse is not supported for: {:?}",
                    purse_uref_value
                ))
            }
        };
        result
    }

    pub(crate) async fn query_global_state_for_entity_addr(
        &self,
        address: &Address
    ) -> Result<EntityAddr, String> {
        let value = self
            .query_global_state(address.as_key(), None)
            .await
            .ok_or(format!(
                "Couldn't query global state for address: {:?}",
                address.to_formatted_string()
            ))?;
        match value {
            StoredValue::SmartContract(package) => package
                .current_entity_hash()
                .map(|addr| addr.value())
                .ok_or(format!(
                    "Couldn't get entity addr for address: {:?}",
                    address.to_formatted_string()
                ))
                .map(EntityAddr::SmartContract),
            StoredValue::ContractPackage(package) => package
                .current_contract_hash()
                .map(|hash| hash.value())
                .ok_or(format!(
                    "Couldn't get last version for address: {:?}",
                    address.to_formatted_string()
                ))
                .map(EntityAddr::SmartContract),
            _ => Err(format!(
                "Entity addr for {:?} was incorrect: {:?}",
                address.to_formatted_string(),
                value
            ))
        }
    }

    pub fn new_call_transaction(
        &self,
        caller: Address,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs,
        amount: u64,
    ) -> Option<Transaction> {
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            contract_address.as_package_hash().unwrap(),
            None,
            entry_point,
            TransactionRuntimeParams::VmCasperV1
        );
        let now = Date::new_0();
        let now_str = now.to_iso_string().as_string()?;
        let timestamp = Timestamp::from_str(&now_str).ok()?;

        
        Some(Transaction::V1(
            transaction_builder
                .with_initiator_addr(*caller.as_account_hash().unwrap())
                .with_ttl(TimeDiff::from_seconds(60))
                .with_chain_name("casper-test")
                .with_pricing_mode(PricingMode::PaymentLimited {
                    payment_amount: amount + 1_000_000_000, //self.gas.as_u64(),
                    gas_price_tolerance: 5,
                    standard_payment: true
                })
                .with_timestamp(timestamp)
                .with_runtime_args(runtime_args)
                .build()
                .unwrap_or_else(|e| {
                    crate::js::log(&format!("failed to build call transaction: {:?}", e));
                    panic!("Failed to build call transaction: {:?}", e)
                })
        ))
    }

    pub(crate) async fn put_transaction(
        &self,
        transaction: Transaction
    ) -> Result<TransactionHash, String> {
        let response = casper_client::put_transaction(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            transaction
        )
        .await;

        let deploy_hash = match response {
            Ok(r) => r.result.transaction_hash,
            Err(e) => {
                return match e {
                    casper_client::Error::ResponseIsRpcError {
                        rpc_method, error, ..
                    } => Err(format!(
                        "Failed to put transaction via RPC method {}: {}",
                        rpc_method, error
                    )),
                    _ => Err(format!("Failed to put transaction: {}", e))
                }
            }
        };
        Ok(deploy_hash)
    }
}
