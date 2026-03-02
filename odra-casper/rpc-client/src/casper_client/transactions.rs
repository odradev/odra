//! Transaction building and deployment methods.

use crate::casper_client::transaction_watcher::{TransactionWatch, TransactionWatcher};
use crate::casper_client::Result;
use crate::error::LivenetError::ExecutionError;
use crate::error::LivenetError::RpcRequestError;
use crate::log;
use casper_client::cli::TransactionV1Builder;
use casper_client::put_transaction;
use casper_types::bytesrepr::{Bytes, ToBytes};
use casper_types::execution::ExecutionResultV1::{Failure, Success};
use casper_types::{
    execution::ExecutionResult, runtime_args, PricingMode, RuntimeArgs, Timestamp, Transaction,
    TransactionHash, TransactionRuntimeParams, TransferTarget, U512
};
use odra_core::consts::{
    AMOUNT_ARG, ARGS_ARG, ATTACHED_VALUE_ARG, ENTRY_POINT_ARG, PACKAGE_HASH_ARG,
    PACKAGE_HASH_KEY_NAME_ARG
};
use odra_core::prelude::*;
use odra_core::CallDef;
use std::time::Duration;

/// Transaction-related constants
const TRANSACTION_WAIT_TIME: u64 = 10;
const TRANSACTION_MAX_RETRIES: u64 = 12;

/// Transaction methods implementation for CasperClient.
impl super::CasperClient {
    /// Transfers the specified number of tokens to the given address.
    pub async fn transfer(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> Result<TransactionHash> {
        let transaction = self.new_transfer_transaction(to, amount, timestamp);
        self.put_transaction(transaction).await
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
            .ok_or_else(|| {
                ExecutionError(format!(
                    "Missing required argument: {}",
                    PACKAGE_HASH_KEY_NAME_ARG
                ))
            })?
            .clone()
            .into_t()
            .map_err(|e| {
                ExecutionError(format!(
                    "Failed to parse {} argument: {:?}",
                    PACKAGE_HASH_KEY_NAME_ARG, e
                ))
            })?;

        let transaction =
            self.new_wasm_deploy_transaction(Bytes::from(wasm_bytes), args, timestamp);
        self.put_transaction(transaction).await?;

        let address = self.get_contract_address(&package_hash_key_name).await?;
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

        let hash = address.as_contract_package_hash().ok_or_else(|| {
            ExecutionError(format!(
                "Address {:?} is not a contract package hash. Expected contract address.",
                address.to_formatted_string()
            ))
        })?;
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

        let module_bytes = include_bytes!("../../resources/proxy_caller_with_return.wasm")
            .to_vec()
            .into();

        let transaction = self.new_wasm_deploy_transaction(module_bytes, args, timestamp);
        let watch = self.start_event_watcher().await?;

        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await
        .map_err(|e| match e {
            casper_client::Error::ResponseIsRpcError {
                rpc_method, error, ..
            } => RpcRequestError(
                rpc_method.to_string(),
                error
                    .data
                    .map_or_else(|| "No data".to_string(), |d| d.to_string())
            ),
            _ => ExecutionError(format!("Failed to put transaction: {}", e))
        })?;
        let deploy_hash = response.result.transaction_hash;
        let result = self.wait_for_transaction(deploy_hash, watch).await?;
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

        let transaction = self.new_call_transaction(addr, call_def, timestamp)?;
        let watch = self.start_event_watcher().await?;

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
        let result = self.wait_for_transaction(deploy_hash, watch).await?;
        self.process_transaction(result, deploy_hash).map(|_| {
            ().to_bytes()
                .expect("Couldn't serialize (). This shouldn't happen.")
                .into()
        })
    }

    async fn wait_for_transaction(
        &self,
        transaction_hash: TransactionHash,
        watch: TransactionWatch
    ) -> Result<ExecutionResult> {
        let transaction_hash_str = transaction_hash.to_hex_string();
        let found = watch
            .wait_for_transaction_hash(&transaction_hash_str)
            .await?;

        if !found {
            return Err(ExecutionError(String::from(
                "Events stream ended before transaction was processed."
            )));
        }

        self.fetch_execution_result(transaction_hash).await
    }

    /// Fetches the execution result for a transaction, with retry logic.
    async fn fetch_execution_result(
        &self,
        transaction_hash: TransactionHash
    ) -> Result<ExecutionResult> {
        let transaction_info = self.get_transaction(transaction_hash).await?;

        if let Some(deploy_info) = transaction_info.execution_info {
            if let Some(execution_result) = deploy_info.execution_result {
                return Ok(execution_result);
            }
        }

        // If execution_info is not available yet, wait a bit and retry
        tokio::time::sleep(Duration::from_millis(500)).await;
        let transaction_info = self.get_transaction(transaction_hash).await?;

        if let Some(deploy_info) = transaction_info.execution_info {
            if let Some(execution_result) = deploy_info.execution_result {
                return Ok(execution_result);
            }
        }

        Err(ExecutionError(String::from(
            "Transaction processed but execution result not available."
        )))
    }

    async fn start_event_watcher(&self) -> Result<TransactionWatch> {
        let timeout = Duration::from_secs(TRANSACTION_WAIT_TIME * TRANSACTION_MAX_RETRIES);
        let watcher = TransactionWatcher::new(self.configuration.events_url.clone(), timeout);
        watcher.start_watching().await
    }

    async fn put_transaction(&self, transaction: Transaction) -> Result<TransactionHash> {
        log::debug("[TX] Starting event watcher before sending transaction...");
        let watch = self.start_event_watcher().await?;
        log::debug("[TX] Event watcher ready, now sending transaction...");
        println!("Transaction sent, waiting for execution result...");
        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await
        .map_err(|e| match e {
            casper_client::Error::ResponseIsRpcError {
                rpc_method, error, ..
            } => {
                eprintln!(
                    "[TX] Received RPC error for method {}: {}.",
                    rpc_method, error
                );
                RpcRequestError(
                    rpc_method.to_string(),
                    error
                        .data
                        .map_or_else(|| "No data".to_string(), |d| d.to_string())
                )
            }
            _ => {
                eprintln!("Failed to put transaction: {}.", e);
                ExecutionError(format!("Failed to put transaction: {}", e))
            }
        })?;
        let deploy_hash = response.result.transaction_hash;
        log::debug(format!(
            "[TX] Transaction sent with hash: {}",
            deploy_hash.to_hex_string()
        ));
        let result = self.wait_for_transaction(deploy_hash, watch).await?;
        self.process_transaction(result, deploy_hash)?;
        Ok(deploy_hash)
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
                    if let Some(url) = self.configuration.transaction_url(&deploy_hash_str) {
                        log::link(url);
                    }
                    Ok(())
                }
                Some(error_message) => {
                    log::error(format!(
                        "Transaction {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message,
                    ));
                    if let Some(url) = self.configuration.transaction_url(&deploy_hash_str) {
                        log::link(url);
                    }
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
    ) -> Result<Transaction> {
        let package_hash = to.as_package_hash().ok_or_else(|| {
            ExecutionError(format!(
                "Address {:?} is not a package hash. Expected contract address.",
                to.to_formatted_string()
            ))
        })?;
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            package_hash,
            None,
            call_def.entry_point(),
            TransactionRuntimeParams::VmCasperV1
        );
        let transaction_v1 = transaction_builder
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
            .map_err(|e| ExecutionError(format!("Failed to build call transaction: {:?}", e)))?;
        Ok(Transaction::V1(transaction_v1))
    }

    fn pricing_mode(&self) -> PricingMode {
        PricingMode::PaymentLimited {
            payment_amount: self.gas.as_u64(),
            gas_price_tolerance: self.configuration.gas_price_tolerance(),
            standard_payment: true
        }
    }
}
