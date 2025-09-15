use crate::{
    now,
    types::{
        Address as WasmAddress, Bytes as WasmBytes, PublicKey,
        TransactionHash as WasmTransactionHash, Verbosity, U512 as WasmU512
    },
    wallet::CasperWallet,
    PROXY_CALLER
};
use casper_client::{
    cli::{DeployBuilder, TransactionV1Builder},
    rpcs::GlobalStateIdentifier,
    JsonRpcId
};
use casper_types::{
    bytesrepr::{Bytes, ToBytes},
    execution::{Effects, TransformKindV2},
    runtime_args, CLValue, Deploy, Digest, EntityAddr, ExecutableDeployItem, Key, PricingMode,
    RuntimeArgs, SecretKey, StoredValue, TimeDiff, Transaction, TransactionHash,
    TransactionRuntimeParams, TransferTarget, URef, U512
};
use odra_core::prelude::Address;
use wasm_bindgen::prelude::*;

const DEFAULT_GAS: u64 = 2_500_000_000;
const DEFAULT_TTL: u32 = 5 * 60;
const DEFAULT_GAS_TOLERANCE: u8 = 5;
const CHAIN_TESTNET: &str = "casper-test";
const SK_STRING: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIODIFIJtQQHcpRuDU0QdaygC/se2mntLKUMK2kCnEsKN
-----END PRIVATE KEY-----"#;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct OdraWasmClient {
    node_address: String,
    speculative_node_address: String,
    verbosity: Verbosity,
    gas: u64,
    chain_name: String,
    ttl: u32
}

#[wasm_bindgen]
impl OdraWasmClient {
    #[wasm_bindgen(constructor)]
    pub fn new(
        node_address: String,
        speculative_node_address: String,
        chain_name: Option<String>,
        gas: Option<u64>,
        ttl: Option<u32>,
        verbosity: Option<Verbosity>
    ) -> Self {
        OdraWasmClient {
            node_address,
            speculative_node_address,
            verbosity: verbosity.unwrap_or(Verbosity::Low),
            gas: gas.unwrap_or(DEFAULT_GAS),
            chain_name: chain_name.unwrap_or(CHAIN_TESTNET.into()),
            ttl: ttl.unwrap_or(DEFAULT_TTL)
        }
    }

    /// Sets the gas limit for the client.
    #[wasm_bindgen(js_name = "setGas")]
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas;
    }

    /// Returns the default payment amount for transactions.
    #[wasm_bindgen(js_name = "DEFAULT_PAYMENT")]
    pub fn default_payment() -> u64 {
        2_500_000_000
    }

    /// Returns the balance of the specified address.
    #[wasm_bindgen(js_name = "getBalance")]
    pub async fn get_balance_js(&self, address: &WasmAddress) -> Result<WasmU512, JsError> {
        self.get_balance((*address).into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e))
    }

    /// Returns the balance of the specified address.
    #[wasm_bindgen(js_name = "getCallerBalance")]
    pub async fn get_caller_balance(&self, wallet: &CasperWallet) -> Result<WasmU512, JsError> {
        let caller = self.caller(wallet).await?;
        self.get_balance(caller.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e))
    }

    /// Returns the address of the caller.
    #[wasm_bindgen(js_name = "caller")]
    pub async fn caller(&self, wallet: &CasperWallet) -> Result<WasmAddress, JsError> {
        let pk_string = wallet.get_active_public_key().await?;
        PublicKey::new(&pk_string)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(Into::<WasmAddress>::into)
    }

    /// Transfers the specified amount to the given address.
    #[wasm_bindgen(js_name = "transfer")]
    pub async fn transfer(
        &self,
        to: &WasmAddress,
        amount: &WasmU512,
        wallet: &CasperWallet
    ) -> Result<WasmTransactionHash, JsError> {
        let caller = self.caller(wallet).await?;
        let transaction: Transaction = self
            .new_transfer_transaction(*caller, **to, **amount)
            .map_err(|e| JsError::new(&format!("Failed to create transaction: {}", e)))?;

        let signed_transaction = wallet.sign_transaction(transaction.into(), None).await?;

        self.put_transaction(signed_transaction.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}

impl OdraWasmClient {
    fn node_address(&self) -> &str {
        &self.node_address
    }

    fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    fn rpc_id(&self) -> JsonRpcId {
        JsonRpcId::String("1".to_string())
    }

    fn pricing_mode(&self) -> PricingMode {
        PricingMode::PaymentLimited {
            payment_amount: self.gas,
            gas_price_tolerance: DEFAULT_GAS_TOLERANCE,
            standard_payment: true
        }
    }
}

impl OdraWasmClient {
    pub async fn call_entry_point(
        &self,
        wallet: &CasperWallet,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<WasmTransactionHash, JsError> {
        let caller = self.caller(wallet).await?;
        let transaction: Transaction = self
            .new_call_transaction(*caller, contract_address, entry_point, runtime_args)
            .map_err(|e| JsError::new(&format!("Failed to create transaction: {}", e)))?;

        let signed_transaction = wallet.sign_transaction(transaction.into(), None).await?;

        self.put_transaction(signed_transaction.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[allow(deprecated)]
    pub async fn call_entry_point_with_proxy(
        &self,
        address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<CLValue, JsError> {
        let hash = address.as_contract_package_hash().ok_or_else(|| {
            JsError::new(&format!(
                "Address is not a contract package hash: {:?}",
                address.to_formatted_string()
            ))
        })?;
        let args_bytes: Vec<u8> = runtime_args
            .to_bytes()
            .map_err(|e| JsError::new(&format!("Failed to serialize runtime args: {}", e)))?;
        let args = runtime_args! {
            "package_hash" => hash,
            "entry_point" => entry_point,
            "args" => Bytes::from(args_bytes),
            "attached_value" => U512::zero(),
            "amount" => U512::zero(),
        };

        let sk = SecretKey::from_pem(SK_STRING).map_err(|e| JsError::new(&e.to_string()))?;
        let signed_deploy = self.new_proxy_deploy(&sk, args).await?;
        let response = casper_client::speculative_exec(
            self.rpc_id(),
            &self.speculative_node_address,
            self.verbosity().into(),
            signed_deploy
        )
        .await;

        let res = response
            .map_err(|e| JsError::new(&format!("Failed to call entry point: {}", e)))?
            .result
            .execution_result;

        find_result(&res.effects).ok_or_else(|| {
            JsError::new(&format!(
                "Failed to find result in effects: {:?}",
                res.effects
            ))
        })
    }

    #[allow(deprecated)]
    pub async fn call_payable_entry_point(
        &self,
        wallet: &CasperWallet,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs,
        attached_value: U512
    ) -> Result<WasmTransactionHash, JsError> {
        let caller = self.caller(wallet).await?;
        let hash = contract_address.as_contract_package_hash().ok_or_else(|| {
            JsError::new(&format!(
                "Address is not a contract package hash: {:?}",
                contract_address.to_formatted_string()
            ))
        })?;
        let args_bytes: Vec<u8> = runtime_args
            .to_bytes()
            .map_err(|e| JsError::new(&format!("Failed to serialize runtime args: {}", e)))?;
        let args = runtime_args! {
            "package_hash" => hash,
            "entry_point" => entry_point,
            "args" => Bytes::from(args_bytes),
            "attached_value" => attached_value,
            "amount" => attached_value,
        };
        let transaction: Transaction = self
            .new_proxy_transaction(*caller, args)
            .map_err(|e| JsError::new(&format!("Failed to create transaction: {}", e)))?;

        let signed_transaction = wallet.sign_transaction(transaction.into(), None).await?;

        self.put_transaction(signed_transaction.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}

impl OdraWasmClient {
    /// Gets a value from a named key of an account or a contract
    pub async fn get_named_value(&self, address: &WasmAddress, name: &str) -> Option<WasmBytes> {
        let entity_hash = self
            .query_global_state_for_entity_addr(address)
            .await
            .ok()?;
        let stored_value = self
            .query_global_state(Key::Hash(entity_hash.value()), Some(name.to_string()))
            .await;
        match stored_value {
            None => None,
            Some(value) => match value {
                StoredValue::CLValue(value) => {
                    Some(WasmBytes::from(value.inner_bytes().as_slice()))
                }
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
}

impl OdraWasmClient {
    pub(crate) async fn get_state_root_hash(&self) -> Result<Option<Digest>, String> {
        casper_client::get_state_root_hash(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            None
        )
        .await
        .map(|r| r.result.state_root_hash)
        .map_err(|_| {
            format!(
                "Couldn't get state root hash from node: {:?}",
                self.node_address()
            )
        })
    }

    async fn query_global_state(&self, key: Key, path: Option<String>) -> Option<StoredValue> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let digest = self.get_state_root_hash().await.ok().flatten()?;
        let result = casper_client::query_global_state(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            GlobalStateIdentifier::StateRootHash(digest),
            key,
            path
        )
        .await;
        match result {
            Ok(r) => Some(r.result.stored_value),
            Err(_) => None
        }
    }

    /// Gets an uref for a main purse of an account or a contract.
    async fn get_main_purse(&self, address: &Address) -> Result<URef, String> {
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
                        .and_then(|v| v.into_uref())
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

    async fn query_global_state_for_entity_addr(
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

    async fn put_transaction(&self, transaction: Transaction) -> Result<TransactionHash, String> {
        let response = casper_client::put_transaction(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            transaction
        )
        .await;

        Ok(match response {
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
        })
    }

    async fn get_balance(&self, address: Address) -> Result<U512, String> {
        let state_root_hash = self
            .get_state_root_hash()
            .await
            .map_err(|err| format!("Error getting state root hash: {err:?}"))?
            .ok_or("State root hash is None, cannot get balance")?;

        let purse = self
            .get_main_purse(&address)
            .await
            .map_err(|err| format!("Error getting main purse: {err:?}"))?;

        let result = casper_client::get_balance(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            state_root_hash,
            purse
        )
        .await
        .map_err(|e| format!("Error getting balance: {e:?}"))?;

        Ok(result.result.balance_value)
    }

    fn new_call_transaction(
        &self,
        caller: Address,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<Transaction, String> {
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            contract_address
                .as_package_hash()
                .ok_or("Invalid contract address")?,
            None,
            entry_point,
            TransactionRuntimeParams::VmCasperV1
        );
        let timestamp = now().ok_or("Failed to get current time")?;

        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(*caller.as_account_hash().ok_or("Invalid caller address")?)
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(timestamp)
                .with_runtime_args(runtime_args)
                .build()
                .map_err(|e| {
                    crate::js::log(&format!("failed to build call transaction: {:?}", e));
                    format!("Failed to build call transaction: {:?}", e)
                })?
        ))
    }

    async fn new_proxy_deploy(&self, sk: &SecretKey, args: RuntimeArgs) -> Result<Deploy, JsError> {
        let proxy_bytes = PROXY_CALLER.to_vec().into();
        DeployBuilder::new(
            &self.chain_name,
            ExecutableDeployItem::ModuleBytes {
                module_bytes: proxy_bytes,
                args
            }
        )
        .with_ttl(TimeDiff::from_seconds(self.ttl))
        .with_account(sk.into())
        .with_timestamp(now().ok_or_else(|| JsError::new("Failed to get current time"))?)
        .with_payment(ExecutableDeployItem::ModuleBytes {
            module_bytes: Default::default(),
            args: runtime_args! {
                "amount" => U512::from(self.gas)
            }
        })
        .with_secret_key(sk)
        .build()
        .map_err(|e| JsError::new(&e.to_string()))
    }

    fn new_transfer_transaction(
        &self,
        caller: Address,
        to: Address,
        amount: U512
    ) -> Result<Transaction, String> {
        let transaction_builder = TransactionV1Builder::new_transfer(
            amount,
            None,
            TransferTarget::AccountHash(*to.as_account_hash().ok_or("Invalid account hash")?),
            None
        )
        .map_err(|e| format!("Failed to build transfer transaction: {:?}", e))?;

        let timestamp = now().ok_or("Failed to get current time")?;
        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(*caller.as_account_hash().ok_or("Invalid caller address")?)
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(timestamp)
                .build()
                .map_err(|e| format!("Failed to build transfer transaction: {:?}", e))?
        ))
    }

    fn new_proxy_transaction(
        &self,
        caller: Address,
        args: RuntimeArgs
    ) -> Result<Transaction, String> {
        let proxy_bytes = PROXY_CALLER.to_vec().into();
        let transaction_builder = TransactionV1Builder::new_session(
            true,
            proxy_bytes,
            TransactionRuntimeParams::VmCasperV1
        );
        let timestamp = now().ok_or("Failed to get current time")?;
        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(*caller.as_account_hash().ok_or("Invalid caller address")?)
                .with_runtime_args(args)
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(timestamp)
                .build()
                .map_err(|e| {
                    crate::js::log(&format!("failed to build call transaction: {:?}", e));
                    format!("Failed to build call transaction: {:?}", e)
                })?
        ))
    }
}

fn find_result(effects: &Effects) -> Option<CLValue> {
    let values = effects.clone().value();
    values.iter().find_map(|effect| {
        if let TransformKindV2::Write(StoredValue::CLValue(cl_value)) = effect.kind() {
            return Some(cl_value.clone());
        }
        None
    })
}
