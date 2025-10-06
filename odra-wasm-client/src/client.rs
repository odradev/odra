use std::{
    str::FromStr,
    sync::{Arc, Mutex, OnceLock}
};

use crate::{
    cspr_click::{get_account, AccountInfo, CsprClick, TransactionResult},
    types::{
        Address as WasmAddress, IntoWasmValue, PublicKey, TransactionHash as WasmTransactionHash,
        Verbosity, U512 as WasmU512
    },
    PROXY_CALLER
};
use casper_client::{
    cli::{DeployBuilder, TransactionV1Builder},
    rpcs::GlobalStateIdentifier,
    JsonRpcId
};
use casper_types::{
    bytesrepr::{Bytes, FromBytes, ToBytes},
    execution::{Effects, TransformKindV2},
    runtime_args, CLValue, Deploy, Digest, ExecutableDeployItem, Key, PricingMode, RuntimeArgs,
    SecretKey, StoredValue, TimeDiff, Timestamp, Transaction, TransactionHash,
    TransactionRuntimeParams, TransferTarget, URef, U512
};
use js_sys::Date;
use odra_core::prelude::Address;
use thiserror::Error;
use wasm_bindgen::prelude::*;

const ARG_PACKAGE_HASH: &str = "package_hash";
const ARG_ENTRY_POINT: &str = "entry_point";
const ARG_ARGS: &str = "args";
const ARG_ATTACHED_VALUE: &str = "attached_value";
const ARG_AMOUNT: &str = "amount";

const DEFAULT_GAS: u64 = 2_500_000_000;
const DEFAULT_TTL: u32 = 5 * 60;
const DEFAULT_GAS_TOLERANCE: u8 = 5;
const CHAIN_TESTNET: &str = "casper-test";
const SECRET_KEY_PEM: &str = env!("WASM_CLIENT_SK");

static GAS: OnceLock<Arc<Mutex<u64>>> = OnceLock::new();

/// Returns the gas limit for the client for the next calls.
#[wasm_bindgen]
pub fn gas() -> u64 {
    *GAS.get_or_init(|| Arc::new(Mutex::new(DEFAULT_GAS)))
        .lock()
        .unwrap()
}

/// Sets the gas limit for the client for the next calls.
#[wasm_bindgen(js_name = "setGas")]
pub fn set_gas(gas: u64) {
    let g = GAS.get_or_init(|| Arc::new(Mutex::new(DEFAULT_GAS)));
    let mut value = g.lock().unwrap();
    *value = gas;
}

/// Returns the default payment amount for transactions.
#[wasm_bindgen(js_name = "DEFAULT_PAYMENT_AMOUNT")]
pub fn default_payment() -> u64 {
    2_500_000_000
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct OdraWasmClient {
    node_address: String,
    speculative_node_address: String,
    verbosity: Verbosity,
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
        ttl: Option<u32>,
        verbosity: Option<Verbosity>
    ) -> Self {
        OdraWasmClient {
            node_address,
            speculative_node_address,
            verbosity: verbosity.unwrap_or(Verbosity::Low),
            chain_name: chain_name.unwrap_or(CHAIN_TESTNET.into()),
            ttl: ttl.unwrap_or(DEFAULT_TTL)
        }
    }

    /// Returns the balance of the specified address.
    #[wasm_bindgen(js_name = "getBalance")]
    pub async fn get_balance_js(&self, address: &WasmAddress) -> Result<WasmU512, JsError> {
        self.get_balance((*address).into()).await.map(Into::into)
    }

    /// Returns the balance of the specified address.
    #[wasm_bindgen(js_name = "getCallerBalance")]
    pub async fn get_caller_balance(&self) -> Result<WasmU512, JsError> {
        let caller = CsprClick::caller().await?;
        self.get_balance(caller.into()).await.map(Into::into)
    }

    /// Returns the balance of the specified address.
    #[wasm_bindgen(js_name = "caller")]
    pub async fn get_caller(&self) -> Result<WasmAddress, JsError> {
        CsprClick::caller().await.map(Into::into)
    }

    /// Transfers the specified amount to the given address.
    #[wasm_bindgen(js_name = "transfer")]
    pub async fn transfer(
        &self,
        to: &WasmAddress,
        amount: &WasmU512
    ) -> Result<WasmTransactionHash, JsError> {
        let caller = CsprClick::caller().await?;
        let transaction: Transaction = self.new_transfer_transaction(*caller, **to, **amount)?;
        let signed_transaction = self.sign_transaction(transaction).await?;
        self.put_transaction(signed_transaction)
            .await
            .map(Into::into)
    }

    // #[wasm_bindgen(js_name = "connect")]
    // pub async fn sign_in(&self) -> Result<(), JsError> {
    //     CsprClick::c().await
    // }

    #[wasm_bindgen(js_name = "signIn")]
    pub async fn sign_in(&self) -> Result<(), JsError> {
        CsprClick::sign_in().await
    }

    #[wasm_bindgen(js_name = "signOut")]
    pub async fn sign_out(&self) -> Result<(), JsError> {
        CsprClick::sign_out().await
    }

    #[wasm_bindgen(js_name = "disconnect")]
    pub async fn disconnect_from_site(&self) -> Result<bool, JsError> {
        CsprClick::disconnect().await
    }

    #[wasm_bindgen(js_name = "signInWithAccount")]
    pub async fn sign_in_with_account(
        &self,
        account: &AccountInfo
    ) -> Result<AccountInfo, JsError> {
        CsprClick::sign_in_with_account(account.clone()).await
    }

    #[wasm_bindgen(js_name = "isUnlocked")]
    pub async fn is_unlocked(&self, provider: &str) -> Result<bool, JsError> {
        CsprClick::is_unlocked(provider).await
    }

    #[wasm_bindgen(js_name = "getActivePublicKey")]
    pub async fn get_active_public_key(&self) -> Result<String, JsError> {
        CsprClick::get_active_public_key().await
    }

    #[wasm_bindgen(js_name = "getActiveAccount")]
    pub async fn get_active_account(&self) -> Result<AccountInfo, JsError> {
        CsprClick::get_active_account().await
    }

    #[wasm_bindgen(js_name = "switchAccount")]
    pub async fn request_switch_account(&self) -> Result<(), JsError> {
        CsprClick::switch_account().await
    }

    #[wasm_bindgen(js_name = "signMessage")]
    pub async fn sign_message(&self, _message: String) -> Result<String, JsError> {
        todo!("Not implemented yet")
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
            payment_amount: gas(),
            gas_price_tolerance: DEFAULT_GAS_TOLERANCE,
            standard_payment: true
        }
    }

    fn caller_and_public_key(&self) -> Result<(Address, String), JsError> {
        let public_key = get_account()?.public_key;
        let caller = PublicKey::new(&public_key)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(Into::<WasmAddress>::into)?
            .into();
        Ok((caller, public_key))
    }

    async fn sign_transaction(&self, transaction: Transaction) -> Result<Transaction, JsError> {
        CsprClick::sign_transaction(transaction.into())
            .await
            .map(Into::into)
    }
}

impl OdraWasmClient {
    pub async fn call_entry_point(
        &self,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<TransactionResult, JsError> {
        let (caller, public_key) = self.caller_and_public_key()?;
        let transaction =
            self.new_call_transaction(caller, contract_address, entry_point, runtime_args)?;
        CsprClick::send_transaction(transaction.into(), public_key).await
    }

    #[allow(deprecated)]
    pub async fn call_entry_point_with_proxy<R, T: FromBytes + IntoWasmValue<R>>(
        &self,
        address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<R, JsError> {
        let hash = address
            .as_contract_package_hash()
            .ok_or(ClientError::InvalidContractAddress(address))?;
        let args_bytes: Vec<u8> = runtime_args.to_bytes()?;
        let args = runtime_args! {
            ARG_PACKAGE_HASH => hash,
            ARG_ENTRY_POINT => entry_point,
            ARG_ARGS => Bytes::from(args_bytes),
            ARG_ATTACHED_VALUE => U512::zero(),
            ARG_AMOUNT => U512::zero(),
        };

        let sk = SecretKey::from_pem(SECRET_KEY_PEM)?;
        let signed_deploy = self.new_proxy_deploy(&sk, args).await?;
        let result = casper_client::speculative_exec(
            self.rpc_id(),
            &self.speculative_node_address,
            self.verbosity().into(),
            signed_deploy
        )
        .await?
        .result
        .execution_result;

        let cl_value = find_result(&result.effects)?;
        let (result, _) = T::from_bytes(&cl_value.inner_bytes()[4..])?;
        Ok(result.to_wasm_value())
    }

    #[allow(deprecated)]
    pub async fn call_payable_entry_point(
        &self,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs,
        attached_value: U512
    ) -> Result<TransactionResult, JsError> {
        let (caller, public_key) = self.caller_and_public_key()?;
        let hash = contract_address
            .as_contract_package_hash()
            .ok_or(ClientError::InvalidContractAddress(contract_address))?;
        let args_bytes: Vec<u8> = runtime_args.to_bytes()?;
        let args = runtime_args! {
            ARG_PACKAGE_HASH => hash,
            ARG_ENTRY_POINT => entry_point,
            ARG_ARGS => Bytes::from(args_bytes),
            ARG_ATTACHED_VALUE => attached_value,
            ARG_AMOUNT => attached_value,
        };
        let transaction = self.new_proxy_transaction(caller, args)?;
        CsprClick::send_transaction(transaction.into(), public_key).await
    }
}

impl OdraWasmClient {
    async fn get_state_root_hash(&self) -> Result<Digest, JsError> {
        casper_client::get_state_root_hash(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            None
        )
        .await?
        .result
        .state_root_hash
        .ok_or(JsError::new("State root hash is None"))
    }

    async fn query_global_state(
        &self,
        key: Key,
        path: Option<String>
    ) -> Result<StoredValue, JsError> {
        let path = match path {
            None => vec![],
            Some(string) => vec![string]
        };
        let digest = self.get_state_root_hash().await?;
        Ok(casper_client::query_global_state(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            GlobalStateIdentifier::StateRootHash(digest),
            key,
            path
        )
        .await?
        .result
        .stored_value)
    }

    /// Gets an uref for a main purse of an account or a contract.
    async fn get_main_purse(&self, address: &Address) -> Result<URef, JsError> {
        let purse_uref_value = self.query_global_state(address.as_key(), None).await?;

        let result = match purse_uref_value {
            StoredValue::CLValue(value) => value.into_t().map_err(|_| {
                ClientError::ReadGlobalStateError(format!(
                    "Couldn't get CLValue for address: {:?}",
                    address.to_formatted_string()
                ))
            }),
            StoredValue::AddressableEntity(entity) => Ok(entity.main_purse()),
            StoredValue::Account(account) => Ok(account.main_purse()),
            StoredValue::ContractPackage(contract_package) => {
                let last_version = contract_package.current_contract_hash().ok_or(
                    ClientError::ReadGlobalStateError(format!(
                        "Couldn't get last version for address: {:?}",
                        address.to_formatted_string()
                    ))
                )?;
                let maybe_contract = self
                    .query_global_state(Key::Hash(last_version.value()), None)
                    .await?;
                match maybe_contract {
                    StoredValue::Contract(contract) => contract
                        .named_keys()
                        .get("__contract_main_purse")
                        .and_then(|v| v.into_uref())
                        .ok_or(ClientError::ReadGlobalStateError(format!(
                            "Couldn't get main purse for address: {:?}",
                            address.to_formatted_string()
                        ))),
                    _ => Err(ClientError::ReadGlobalStateError(format!(
                        "Couldn't get main purse for address: {:?}",
                        address.to_formatted_string()
                    )))
                }
            }
            _ => Err(ClientError::ReadGlobalStateError(format!(
                "Getting main purse is not supported for: {:?}",
                purse_uref_value
            )))
        }?;
        Ok(result)
    }

    async fn put_transaction(&self, transaction: Transaction) -> Result<TransactionHash, JsError> {
        Ok(casper_client::put_transaction(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            transaction
        )
        .await?
        .result
        .transaction_hash)
    }

    async fn get_balance(&self, address: Address) -> Result<U512, JsError> {
        let state_root_hash = self.get_state_root_hash().await?;

        let purse = self.get_main_purse(&address).await?;

        let result = casper_client::get_balance(
            self.rpc_id(),
            self.node_address(),
            self.verbosity().into(),
            state_root_hash,
            purse
        )
        .await?;

        Ok(result.result.balance_value)
    }

    fn new_call_transaction(
        &self,
        caller: Address,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<Transaction, JsError> {
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            contract_address
                .as_package_hash()
                .ok_or(ClientError::InvalidAccountAddress(contract_address))?,
            None,
            entry_point,
            TransactionRuntimeParams::VmCasperV1
        );
        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(
                    *caller
                        .as_account_hash()
                        .ok_or(ClientError::InvalidAccountAddress(caller))?
                )
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(now()?)
                .with_runtime_args(runtime_args)
                .build()?
        ))
    }

    async fn new_proxy_deploy(&self, sk: &SecretKey, args: RuntimeArgs) -> Result<Deploy, JsError> {
        let proxy_bytes = PROXY_CALLER.to_vec().into();
        Ok(DeployBuilder::new(
            &self.chain_name,
            ExecutableDeployItem::ModuleBytes {
                module_bytes: proxy_bytes,
                args
            }
        )
        .with_ttl(TimeDiff::from_seconds(self.ttl))
        .with_account(sk.into())
        .with_timestamp(now()?)
        .with_payment(ExecutableDeployItem::ModuleBytes {
            module_bytes: Default::default(),
            args: runtime_args! {
                "amount" => U512::from(gas())
            }
        })
        .with_secret_key(sk)
        .build()?)
    }

    fn new_transfer_transaction(
        &self,
        caller: Address,
        to: Address,
        amount: U512
    ) -> Result<Transaction, JsError> {
        let transaction_builder = TransactionV1Builder::new_transfer(
            amount,
            None,
            TransferTarget::AccountHash(
                *to.as_account_hash()
                    .ok_or(ClientError::InvalidAccountAddress(to))?
            ),
            None
        )
        .map_err(ClientError::CLValue)?;

        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(
                    *caller
                        .as_account_hash()
                        .ok_or(ClientError::InvalidAccountAddress(caller))?
                )
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(now()?)
                .build()?
        ))
    }

    fn new_proxy_transaction(
        &self,
        caller: Address,
        args: RuntimeArgs
    ) -> Result<Transaction, JsError> {
        let proxy_bytes = PROXY_CALLER.to_vec().into();
        let transaction_builder = TransactionV1Builder::new_session(
            true,
            proxy_bytes,
            TransactionRuntimeParams::VmCasperV1
        );
        Ok(Transaction::V1(
            transaction_builder
                .with_initiator_addr(
                    *caller
                        .as_account_hash()
                        .ok_or(ClientError::InvalidAccountAddress(caller))?
                )
                .with_runtime_args(args)
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(now()?)
                .build()?
        ))
    }
}

fn find_result(effects: &Effects) -> Result<CLValue, JsError> {
    let values = effects.clone().value();
    values
        .iter()
        .find_map(|effect| {
            if let TransformKindV2::Write(StoredValue::CLValue(cl_value)) = effect.kind() {
                return Some(cl_value.clone());
            }
            None
        })
        .ok_or_else(|| JsError::new("Couldn't find CLValue in the execution result"))
}

fn now() -> Result<Timestamp, JsError> {
    let now = Date::new_0();
    let now_str = now
        .to_iso_string()
        .as_string()
        .ok_or_else(|| JsError::new("Failed to convert date to string"))?;
    let timestamp = Timestamp::from_str(&now_str)?;
    Ok(timestamp)
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Invalid account address: {:?}", .0.to_formatted_string())]
    InvalidAccountAddress(Address),
    #[error("Address is not a contract package hash: {:?}", .0.to_formatted_string())]
    InvalidContractAddress(Address),
    #[error("Failed to serialize/deserialize CLValue: {0}")]
    CLValue(casper_types::CLValueError),
    #[error("Read global state error: {0}")]
    ReadGlobalStateError(String)
}
