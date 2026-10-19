//! Transaction building and deployment methods.

use crate::casper_client::transaction_watcher::ProcessedTransaction;
use crate::casper_client::Result;
use crate::error::LivenetError;
use crate::log;
use crate::utils::block_on;
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
use odra_core::{CallDef, DeployReport};

/// Gas amount used for native transfers.
const NATIVE_TRANSFER_GAS: u64 = 100_000_000u64;

/// Transaction methods implementation for CasperClient.
impl super::CasperClient {
    /// Transfers the specified number of tokens to the given address.
    pub fn transfer(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> Result<TransactionHash> {
        block_on(self.transfer_async(to, amount, timestamp))
    }

    /// Transfers the specified number of tokens to the given address.
    pub async fn transfer_async(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> Result<TransactionHash> {
        let transaction = self.new_transfer_transaction(to, amount, timestamp)?;
        log::debug(serde_json::to_string_pretty(&transaction).unwrap());
        self.put_transaction(transaction, GasEntry::Transfer).await
    }

    /// Deploy the contract.
    pub fn deploy_wasm(
        &self,
        contract_name: &str,
        args: RuntimeArgs,
        timestamp: Timestamp,
        wasm_bytes: Vec<u8>
    ) -> Result<Address> {
        block_on(self.deploy_wasm_async(contract_name, args, timestamp, wasm_bytes))
    }

    /// Deploy the contract.
    pub async fn deploy_wasm_async(
        &self,
        contract_name: &str,
        args: RuntimeArgs,
        timestamp: Timestamp,
        wasm_bytes: Vec<u8>
    ) -> Result<Address> {
        log::info(format!("Deploying \"{}\".", contract_name));

        let package_hash_key_name: String = args
            .get(PACKAGE_HASH_KEY_NAME_ARG)
            .ok_or_else(|| {
                LivenetError::ExecutionError(format!(
                    "Missing required argument: {}",
                    PACKAGE_HASH_KEY_NAME_ARG
                ))
            })?
            .clone()
            .into_t()
            .map_err(|e| {
                LivenetError::ExecutionError(format!(
                    "Failed to parse {} argument: {:?}",
                    PACKAGE_HASH_KEY_NAME_ARG, e
                ))
            })?;

        if self.gas.is_zero() {
            return Err(LivenetError::GasNotSet);
        }

        let transaction =
            self.new_wasm_deploy_transaction(Bytes::from(wasm_bytes), args, timestamp)?;
        log::debug(serde_json::to_string_pretty(&transaction).unwrap());
        let entry = GasEntry::WasmDeploy(format!("{}.wasm", contract_name));
        self.put_transaction(transaction, entry).await?;

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
    pub fn deploy_entrypoint_call_with_proxy(
        &self,
        address: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Result<Bytes> {
        block_on(self.deploy_entrypoint_call_with_proxy_async(address, call_def, timestamp))
    }

    /// Deploy the entrypoint call using getter_proxy.
    /// It runs the getter_proxy contract in an account context and stores the return value of the call
    /// in under the key RESULT_KEY.
    pub async fn deploy_entrypoint_call_with_proxy_async(
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
            LivenetError::ExecutionError(format!(
                "Address {:?} is not a contract package hash. Expected contract address.",
                address.to_formatted_string()
            ))
        })?;
        let args_bytes: Vec<u8> = call_def
            .args()
            .to_bytes()
            .expect("Should serialize to bytes");
        let entry_point = call_def.entry_point();
        let entry = GasEntry::ContractCall(address, call_def.clone());
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

        let transaction = self.new_wasm_deploy_transaction(module_bytes, args, timestamp)?;
        log::debug(serde_json::to_string_pretty(&transaction).unwrap());
        self.ensure_not_pinned()?;
        let watch = self.watcher.start_watching().await?;

        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await
        .map_err(put_transaction_error)?;
        let transaction_hash = response.result.transaction_hash;
        let result = watch.wait_for_transaction_hash(&transaction_hash).await?;
        self.process_transaction(result, transaction_hash, entry)?;
        Ok(self.get_proxy_result().await)
    }

    /// Deploy the entrypoint call.
    pub fn deploy_entrypoint_call(
        &self,
        addr: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Result<Bytes> {
        block_on(self.deploy_entrypoint_call_async(addr, call_def, timestamp))
    }

    /// Deploy the entrypoint call.
    pub async fn deploy_entrypoint_call_async(
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

        let entry = GasEntry::ContractCall(addr, call_def.clone());
        let transaction = self.new_call_transaction(addr, call_def, timestamp)?;
        log::debug(serde_json::to_string_pretty(&transaction).unwrap());
        self.ensure_not_pinned()?;
        let watch = self.watcher.start_watching().await?;

        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await;
        let transaction_hash = response
            .map_err(put_transaction_error)?
            .result
            .transaction_hash;
        let result = watch.wait_for_transaction_hash(&transaction_hash).await?;
        self.process_transaction(result, transaction_hash, entry)
            .map(|_| ().to_bytes().expect("Couldn't serialize (). This shouldn't happen.").into())
    }

    async fn put_transaction(
        &self,
        transaction: Transaction,
        entry: GasEntry
    ) -> Result<TransactionHash> {
        log::debug("[TX] Starting event watcher before sending transaction...");
        self.ensure_not_pinned()?;
        let watch = self.watcher.start_watching().await?;
        log::debug("[TX] Event watcher ready, now sending transaction...");

        let response = put_transaction(
            self.rpc_id_typed(),
            self.configuration.node_address(),
            self.configuration.verbosity_typed(),
            transaction
        )
        .await
        .map_err(put_transaction_error)?;
        let transaction_hash = response.result.transaction_hash;
        log::debug(format!(
            "[TX] Transaction sent with hash: {}",
            transaction_hash.to_hex_string()
        ));
        let result = watch.wait_for_transaction_hash(&transaction_hash).await?;
        self.process_transaction(result, transaction_hash, entry)?;
        Ok(transaction_hash)
    }

    /// Handles the result of an executed transaction: records its messages and gas (a failed
    /// transaction consumed gas too) and turns a failure into an error.
    fn process_transaction(
        &self,
        processed: ProcessedTransaction,
        transaction_hash: TransactionHash,
        entry: GasEntry
    ) -> Result<()> {
        // The transaction changed the global state; the next query must see the new root.
        self.invalidate_state_root_hash();
        self.record_messages(processed.messages);
        let gas = consumed_gas(&processed.execution_result);
        self.record_gas(gas, entry.into_report(gas));
        let deploy_hash_str = transaction_hash.to_hex_string();
        match processed.execution_result {
            ExecutionResult::V1(r) => match r {
                Failure { error_message, .. } => {
                    log::error(format!(
                        "Deploy V1 {:?} failed with error: {:?}.",
                        deploy_hash_str, error_message
                    ));
                    Err(LivenetError::ExecutionError(error_message.to_string()))
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
                    Err(LivenetError::ExecutionError(error_message.to_string()))
                }
            }
        }
    }

    fn new_wasm_deploy_transaction(
        &self,
        transaction_bytes: Bytes,
        args: RuntimeArgs,
        timestamp: Timestamp
    ) -> Result<Transaction> {
        let transaction_builder = TransactionV1Builder::new_session(
            true,
            transaction_bytes,
            TransactionRuntimeParams::VmCasperV1
        );
        Ok(Transaction::V1(
            transaction_builder
                .with_runtime_args(args)
                .with_ttl(self.configuration.ttl())
                .with_chain_name(self.configuration.chain_name())
                .with_pricing_mode(self.pricing_mode())
                .with_secret_key(self.secret_key())
                .with_timestamp(timestamp)
                .build()
                .map_err(|_| LivenetError::InvalidTransaction)?
        ))
    }

    fn new_transfer_transaction(
        &self,
        to: Address,
        amount: U512,
        timestamp: Timestamp
    ) -> Result<Transaction> {
        let target = TransferTarget::AccountHash(
            *to.as_account_hash()
                .ok_or(LivenetError::InvalidTransferTarget(to))?
        );
        let transaction_builder = TransactionV1Builder::new_transfer(amount, None, target, None)
            .map_err(|_| LivenetError::SerializationError)?;
        Ok(Transaction::V1(
            transaction_builder
                .with_ttl(self.configuration.ttl())
                .with_chain_name(self.configuration.chain_name())
                .with_pricing_mode(PricingMode::PaymentLimited {
                    payment_amount: NATIVE_TRANSFER_GAS,
                    gas_price_tolerance: self.configuration.gas_price_tolerance(),
                    standard_payment: true
                })
                .with_secret_key(self.secret_key())
                .with_timestamp(timestamp)
                .build()
                .map_err(|_| LivenetError::InvalidTransaction)?
        ))
    }

    fn new_call_transaction(
        &self,
        to: Address,
        call_def: CallDef,
        timestamp: Timestamp
    ) -> Result<Transaction> {
        let package_hash = to
            .as_package_hash()
            .ok_or(LivenetError::InvalidTransferTarget(to))?;
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
                gas_price_tolerance: self.configuration.gas_price_tolerance(),
                standard_payment: true
            })
            .with_secret_key(self.secret_key())
            .with_timestamp(timestamp)
            .with_runtime_args(call_def.args().clone())
            .build()
            .map_err(|_| LivenetError::InvalidTransaction)?;
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

/// What a transaction did, to name it in the gas report.
enum GasEntry {
    /// A native transfer, not part of the gas report.
    Transfer,
    /// A deploy of the wasm file of the given name: a contract installation or upgrade.
    WasmDeploy(String),
    /// A call of a contract entry point, direct or through the proxy.
    ContractCall(Address, CallDef)
}

impl GasEntry {
    fn into_report(self, gas: U512) -> Option<DeployReport> {
        match self {
            GasEntry::Transfer => None,
            GasEntry::WasmDeploy(file_name) => Some(DeployReport::WasmDeploy { gas, file_name }),
            GasEntry::ContractCall(contract_address, call_def) => Some(DeployReport::ContractCall {
                gas,
                contract_address,
                call_def
            })
        }
    }
}

/// The gas consumed by an executed transaction, in gas units like CasperVM reports it (the
/// motes charged are this times the gas price, plus whatever of the payment limit was not
/// refunded). A legacy (V1) result only carries its cost.
fn consumed_gas(result: &ExecutionResult) -> U512 {
    match result {
        ExecutionResult::V1(Failure { cost, .. }) | ExecutionResult::V1(Success { cost, .. }) => {
            *cost
        }
        ExecutionResult::V2(r) => r.consumed.value()
    }
}

/// Maps a failed `account_put_transaction` call to a [LivenetError].
///
/// The node rejects a transaction from an account that has never received CSPR with a terse
/// "no such addressable entity"; explain what that means, it is the most common first-deploy error.
fn put_transaction_error(e: casper_client::Error) -> LivenetError {
    match e {
        casper_client::Error::ResponseIsRpcError {
            rpc_method, error, ..
        } => {
            let data = error
                .data
                .map_or_else(|| "No data".to_string(), |d| d.to_string());
            let data = if data.contains("no such addressable entity") {
                format!(
                    "{data}. The sending account does not exist on chain yet: an account is \
                     created by the first transfer to it, so fund it with CSPR before deploying"
                )
            } else {
                data
            };
            LivenetError::RpcRequestError(rpc_method.to_string(), data)
        }
        _ => LivenetError::ExecutionError(format!("Failed to put transaction: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::{GasEntry, ProcessedTransaction};
    use crate::casper_client::configuration::CasperClientConfiguration;
    use crate::casper_client::CasperClient;
    use crate::error::LivenetError;
    use casper_types::execution::{Effects, ExecutionResult, ExecutionResultV2};
    use casper_types::{account::AccountHash, Gas, InitiatorAddr, TransactionHash, U512};
    use casper_types::{contracts::ContractPackageHash, RuntimeArgs};
    use odra_core::prelude::*;
    use odra_core::{CallDef, DeployReport};

    const CONTRACT: Address = Address::Contract(ContractPackageHash::new([1; 32]));

    fn client() -> CasperClient {
        CasperClient::new(CasperClientConfiguration {
            node_address: "http://localhost:1".to_string(),
            events_url: "http://localhost:1/events".to_string(),
            chain_name: "casper-net-1".to_string(),
            secret_keys: vec![],
            secret_key_paths: vec![],
            cspr_cloud_auth_token: None,
            gas_price_tolerance: 1,
            ttl: 300,
            state_root_hash: None
        })
    }

    /// An executed transaction that consumed `consumed` gas units out of a limit of 10 000 and
    /// was charged its whole limit.
    fn processed(consumed: u64, error_message: Option<&str>) -> ProcessedTransaction {
        ProcessedTransaction {
            execution_result: ExecutionResult::V2(Box::new(ExecutionResultV2 {
                initiator: InitiatorAddr::AccountHash(AccountHash::new([7; 32])),
                error_message: error_message.map(String::from),
                current_price: 1,
                limit: Gas::new(10_000),
                consumed: Gas::new(consumed),
                cost: U512::from(10_000),
                refund: U512::zero(),
                transfers: vec![],
                size_estimate: 0,
                effects: Effects::new()
            })),
            messages: vec![]
        }
    }

    fn hash() -> TransactionHash {
        TransactionHash::from_raw([2; 32])
    }

    fn call() -> CallDef {
        CallDef::new("increment", true, RuntimeArgs::new())
    }

    #[test]
    fn deploys_and_calls_are_reported_with_the_consumed_gas() {
        let client = client();
        let deploy = GasEntry::WasmDeploy(String::from("Counter.wasm"));
        client
            .process_transaction(processed(1_000, None), hash(), deploy)
            .unwrap();
        let call_entry = GasEntry::ContractCall(CONTRACT, call());
        client
            .process_transaction(processed(300, None), hash(), call_entry)
            .unwrap();
        assert_eq!(client.last_transaction_gas(), U512::from(300));

        let report: Vec<DeployReport> = client.gas_report().into_iter().collect();
        assert_eq!(report.len(), 2);
        match &report[0] {
            DeployReport::WasmDeploy { gas, file_name } => {
                assert_eq!(*gas, U512::from(1_000));
                assert_eq!(file_name, "Counter.wasm");
            }
            other => panic!("unexpected {other:?}")
        }
        match &report[1] {
            DeployReport::ContractCall {
                gas,
                contract_address,
                call_def
            } => {
                assert_eq!(*gas, U512::from(300));
                assert_eq!(*contract_address, CONTRACT);
                assert_eq!(*call_def, call());
            }
            other => panic!("unexpected {other:?}")
        }
    }

    #[test]
    fn failed_call_is_reported_too() {
        let client = client();
        let entry = GasEntry::ContractCall(CONTRACT, call());
        let result =
            client.process_transaction(processed(10_000, Some("Out of gas error")), hash(), entry);
        assert!(matches!(result, Err(LivenetError::ExecutionError(_))));
        assert_eq!(client.last_transaction_gas(), U512::from(10_000));
        assert_eq!(client.gas_report().iter().count(), 1);
    }

    #[test]
    fn transfer_sets_the_last_gas_but_is_not_reported() {
        let client = client();
        client
            .process_transaction(
                processed(500, None),
                hash(),
                GasEntry::ContractCall(CONTRACT, call())
            )
            .unwrap();
        client
            .process_transaction(processed(100, None), hash(), GasEntry::Transfer)
            .unwrap();
        assert_eq!(client.last_transaction_gas(), U512::from(100));
        assert_eq!(client.gas_report().iter().count(), 1);
    }
}
