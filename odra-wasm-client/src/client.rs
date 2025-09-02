use crate::{
    now,
    types::{
        Address as WasmAddress, Bytes as WasmBytes, PublicKey,
        TransactionHash as WasmTransactionHash, Verbosity
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
    TransactionRuntimeParams, URef, U512
};
use odra_core::prelude::Address;
use wasm_bindgen::prelude::*;

const DEFAULT_GAS: u64 = 2_500_000_000;
const DEFAULT_TTL: u32 = 5 * 60;
const DEFAULT_GAS_TOLERANCE: u8 = 5;
const CHAIN_TESTNET: &str = "casper-test";

#[wasm_bindgen]
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

    #[wasm_bindgen(js_name = "setGas")]
    pub fn set_gas(&mut self, gas: u64) {
        self.gas = gas;
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

    async fn caller(&self, wallet: &CasperWallet) -> Result<WasmAddress, JsError> {
        let pk_string = wallet.get_active_public_key().await?;
        let pk = PublicKey::new(&pk_string).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(WasmAddress::from(pk))
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
            .ok_or_else(|| JsError::new("Failed to create transaction"))?;

        let signed_transaction = wallet.sign_transaction(transaction.into(), None).await?;

        self.put_transaction(signed_transaction.into())
            .await
            .map(Into::into)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[allow(deprecated)]
    pub async fn call_entry_point_with_proxy(
        &self,
        wallet: &CasperWallet,
        address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Result<CLValue, JsError> {
        crate::js::log(&format!(
            "Calling entry point '{}' on contract at address: {:?} with args: {:?}",
            entry_point,
            address.to_formatted_string(),
            runtime_args
        ));
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

        let signed_deploy = self.new_proxy_deploy(wallet, args).await?;
        // let signed_deploy = wallet.sign_deploy(deploy.into(), None).await?;
        // let signed_deploy_json = signed_deploy.to_json_string();
        // crate::js::log(&format!("Signed deploy: {}", signed_deploy_json.unwrap()));
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

        crate::js::log(&format!("Wasm effects: {:?}", res));
        let caller = self.caller(wallet).await?;
        find_result(*caller, &res.effects).ok_or_else(|| {
            JsError::new(&format!(
                "Failed to find result in effects: {:?}",
                res.effects
            ))
        })
    }
}

#[wasm_bindgen]
impl OdraWasmClient {
    /// Gets a value from a named key of an account or a contract
    #[wasm_bindgen(js_name = "getNamedValue")]
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

    fn new_call_transaction(
        &self,
        caller: Address,
        contract_address: Address,
        entry_point: &str,
        runtime_args: RuntimeArgs
    ) -> Option<Transaction> {
        let transaction_builder = TransactionV1Builder::new_targeting_package(
            contract_address.as_package_hash().unwrap(),
            None,
            entry_point,
            TransactionRuntimeParams::VmCasperV1
        );
        let timestamp = now()?;

        Some(Transaction::V1(
            transaction_builder
                .with_initiator_addr(*caller.as_account_hash().unwrap())
                .with_ttl(TimeDiff::from_seconds(self.ttl))
                .with_chain_name(&self.chain_name)
                .with_pricing_mode(self.pricing_mode())
                .with_timestamp(timestamp)
                .with_runtime_args(runtime_args)
                .build()
                .unwrap_or_else(|e| {
                    crate::js::log(&format!("failed to build call transaction: {:?}", e));
                    panic!("Failed to build call transaction: {:?}", e)
                })
        ))
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

    pub async fn get_balance(&self, address: Address) -> Result<U512, String> {
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

    async fn new_proxy_deploy(
        &self,
        wallet: &CasperWallet,
        args: RuntimeArgs
    ) -> Result<Deploy, JsError> {
        let proxy_bytes = PROXY_CALLER.to_vec().into();
        let sk_string = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIODIFIJtQQHcpRuDU0QdaygC/se2mntLKUMK2kCnEsKN
-----END PRIVATE KEY-----"#;
        let pk = PublicKey::new(&wallet.get_active_public_key().await?)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let sk = SecretKey::from_pem(sk_string).map_err(|e| JsError::new(&e.to_string()))?;

        DeployBuilder::new(
            &self.chain_name,
            ExecutableDeployItem::ModuleBytes {
                module_bytes: proxy_bytes,
                args
            }
        )
        .with_ttl(TimeDiff::from_seconds(self.ttl))
        .with_account(pk.into())
        .with_timestamp(now().ok_or_else(|| JsError::new("Failed to get current time"))?)
        .with_payment(ExecutableDeployItem::ModuleBytes {
            module_bytes: Default::default(),
            args: runtime_args! {
                "amount" => U512::from(self.gas)
            }
        })
        .with_secret_key(&sk)
        .build()
        .map_err(|e| JsError::new(&e.to_string()))
    }
}

fn find_result(caller: Address, effects: &Effects) -> Option<CLValue> {
    let values = effects.clone().value();
    let result_key = values.iter().find_map(|effect| {
        let k = Address::try_from(*effect.key()).ok();
        let v = effect.kind();
        if k == Some(caller) {
            if let TransformKindV2::AddKeys(nk) = v {
                if let Some(k) = nk.get("__result") {
                    return Some(*k);
                }
            }
        }
        None
    })?;
    let result_uref_addr = result_key.as_uref().map(|uref| uref.addr())?;
    values.iter().find_map(|effect| {
        let effect_uref_addr = effect.key().as_uref().map(|uref| uref.addr());
        if effect_uref_addr == Some(result_uref_addr) {
            if let TransformKindV2::Write(StoredValue::CLValue(cl_value)) = effect.kind() {
                return Some(cl_value.clone());
            }
        }
        None
    })
}
