use odra_casper_shared::consts::*;
use odra_core::prelude::{collections::*, *};

use odra_core::prelude::{collections::*, *};
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT
};
use std::rc::Rc;

use casper_execution_engine::core::engine_state::{GenesisAccount, RunGenesisRequest};
use odra_casper_shared::consts;
use odra_casper_shared::consts::*;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{CallDef, ContractEnv, HostContext, HostEnv};
use odra_types::casper_types::account::AccountHash;
use odra_types::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_types::casper_types::{
    runtime_args, BlockTime, ContractPackageHash, Key, Motes, SecretKey, StoredValue
};
use odra_types::{Address, PublicKey, U512};
use odra_types::{CLTyped, FromBytes, OdraError, RuntimeArgs};

pub struct CasperVm {
    pub accounts: Vec<Address>,
    pub key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    pub active_account: Address,
    pub context: InMemoryWasmTestBuilder,
    pub block_time: BlockTime,
    pub calls_counter: u32,
    pub error: Option<OdraError>,
    pub attached_value: Option<U512>,
    pub gas_used: BTreeMap<AccountHash, U512>,
    pub gas_cost: Vec<(String, U512)>
}

impl CasperVm {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
    }

    fn new_instance() -> Self {
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        let mut accounts: Vec<Address> = Vec::new();
        let mut key_pairs = BTreeMap::new();
        for i in 0..20 {
            // Create keypair.
            let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
            let public_key = PublicKey::from(&secret_key);

            // Create an AccountHash from a public key.
            let account_addr = AccountHash::from(&public_key);

            // Create a GenesisAccount.
            let account = GenesisAccount::account(
                public_key.clone(),
                Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
                None
            );
            genesis_config.ee_config_mut().push_account(account);

            accounts.push(account_addr.try_into().unwrap());
            key_pairs.insert(account_addr.try_into().unwrap(), (secret_key, public_key));
        }
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
            DEFAULT_CHAINSPEC_REGISTRY.clone()
        );

        let chainspec_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/chainspec.toml");
        let mut builder = InMemoryWasmTestBuilder::new_with_chainspec(chainspec_path, None);

        builder.run_genesis(&run_genesis_request).commit();

        Self {
            active_account: accounts[0],
            context: builder,
            accounts,
            block_time: BlockTime::new(0),
            calls_counter: 0,
            error: None,
            attached_value: None,
            gas_used: BTreeMap::new(),
            gas_cost: Vec::new(),
            key_pairs
        }
    }

    fn contract_env(&self) -> ContractEnv {
        todo!()
    }

    fn deploy_contract(&mut self, wasm_path: &str, args: &RuntimeArgs) {
        self.error = None;
        let session_code = PathBuf::from(wasm_path);
        // if let Ok(path) = env::var(ODRA_WASM_PATH_ENV_KEY) {
        //     let mut path = PathBuf::from(path);
        //     path.push(wasm_path);
        //     if path.exists() {
        //         session_code = path;
        //     } else {
        //         panic!("WASM file not found: {:?}", path);
        //     }
        // }

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args.clone())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(u64::from(self.block_time))
            .build();
        self.context.exec(execute_request).commit().expect_success();
        self.collect_gas();
        self.gas_cost.push((
            format!("deploy_contract {}", wasm_path),
            self.last_call_contract_gas_cost()
        ));
    }

    /// Read a ContractPackageHash of a given name, from the active account.
    pub fn contract_package_hash_from_name(&self, name: &str) -> ContractPackageHash {
        let account = self
            .context
            .get_account(self.active_account_hash())
            .unwrap();
        let key: &Key = account.named_keys().get(name).unwrap();
        ContractPackageHash::from(key.into_hash().unwrap())
    }
    pub fn set_caller(&mut self, caller: Address) {
        self.active_account = caller;
    }

    pub fn get_account(&self, index: usize) -> Address {
        self.accounts[index]
    }

    pub fn advance_block_time(&mut self, time_diff: BlockTime) {
        todo!()
    }

    pub fn get_event(
        &self,
        contract_address: Address,
        index: i32
    ) -> Option<odra_types::EventData> {
        todo!()
    }

    pub fn attach_value(&mut self, amount: U512) {
        todo!()
    }

    pub fn call_contract(&mut self, address: &Address, call_def: CallDef) -> Bytes {
        self.error = None;
        // TODO: handle unwrap
        let hash = *address.as_contract_package_hash().unwrap();

        let session_code = include_bytes!("../resources/proxy_caller_with_return.wasm").to_vec();
        let args_bytes: Vec<u8> = call_def.args.to_bytes().unwrap();
        let entry_point = call_def.entry_point.clone();
        let args = runtime_args! {
            CONTRACT_PACKAGE_HASH_ARG => hash,
            ENTRY_POINT_ARG => entry_point,
            ARGS_ARG => Bytes::from(args_bytes),
            ATTACHED_VALUE_ARG => self.attached_value,
            AMOUNT_ARG => self.attached_value.unwrap_or_default(),
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_bytes(session_code, args)
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(u64::from(self.block_time))
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_cost.push((
            format!("call_entrypoint {}", call_def.entry_point),
            self.last_call_contract_gas_cost()
        ));

        self.attached_value = None;
        if let Some(error) = self.context.get_error() {
            // TODO: handle error
            // let odra_error = parse_error(error);
            panic!("Error: {}", error);
            // self.error = Some(odra_error.clone());
            // self.panic_with_error(odra_error, call_def.entry_point, hash);
        } else {
            self.get_active_account_result()
        }
    }

    pub fn new_contract(
        &mut self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        let wasm_path = format!("{}.wasm", name);
        let package_hash_key_name = format!("{}_package_hash", name);
        let mut args = init_args.clone().unwrap_or(runtime_args! {});
        args.insert(PACKAGE_HASH_KEY_NAME_ARG, package_hash_key_name.clone())
            .unwrap();
        args.insert(ALLOW_KEY_OVERRIDE_ARG, true).unwrap();
        args.insert(IS_UPGRADABLE_ARG, false).unwrap();

        if init_args.is_some() {
            args.insert(CONSTRUCTOR_NAME_ARG, CONSTRUCTOR_NAME.to_string())
                .unwrap();
        };

        self.deploy_contract(&wasm_path, &args);
        let contract_package_hash = self.contract_package_hash_from_name(&package_hash_key_name);
        contract_package_hash.try_into().unwrap()
    }
    /// Create a new instance with predefined accounts.
    pub fn active_account_hash(&self) -> AccountHash {
        *self.active_account.as_account_hash().unwrap()
    }

    pub fn next_hash(&mut self) -> [u8; 32] {
        let seed = self.calls_counter;
        self.calls_counter += 1;
        let mut hash = [0u8; 32];
        hash[0] = seed as u8;
        hash[1] = (seed >> 8) as u8;
        hash
    }

    /// Read a value from Account's named keys.
    pub fn get_account_value<T: CLTyped + FromBytes + ToBytes>(
        &self,
        hash: AccountHash,
        name: &str
    ) -> Result<T, String> {
        let result: Result<StoredValue, String> =
            self.context
                .query(None, Key::Account(hash), &[name.to_string()]);

        result.map(|value| value.as_cl_value().unwrap().clone().into_t().unwrap())
    }

    pub fn get_active_account_result(&self) -> Bytes {
        let active_account = self.active_account_hash();
        let bytes: Bytes = self
            .get_account_value(active_account, RESULT_KEY)
            .unwrap_or_default();
        bytes
    }

    pub fn collect_gas(&mut self) {
        *self
            .gas_used
            .entry(*self.active_account.as_account_hash().unwrap())
            .or_insert_with(U512::zero) += *DEFAULT_PAYMENT;
    }

    /// Returns the cost of the last deploy.
    /// Keep in mind that this may be different from the cost of the deploy on the live network.
    /// This is NOT the amount of gas charged - see [last_call_contract_gas_used].
    pub fn last_call_contract_gas_cost(&self) -> U512 {
        self.context.last_exec_gas_cost().value()
    }

    /// Returns the amount of gas used for last call.
    pub fn last_call_contract_gas_used(&self) -> U512 {
        *DEFAULT_PAYMENT
    }

    /// Returns total gas used by the account.
    pub fn total_gas_used(&self, address: Address) -> U512 {
        match &address {
            Address::Account(address) => self.gas_used.get(address).cloned().unwrap_or_default(),
            Address::Contract(address) => panic!("Contract {} can't burn gas.", address)
        }
    }

    /// Returns the report of the gas used during the whole lifetime of the CasperVM.
    pub fn gas_report(&self) -> Vec<(String, U512)> {
        self.gas_cost.clone()
    }

    /// Returns the public key that corresponds to the given Account Address.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }

    /// Cryptographically signs a message as a given account.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        let (secret_key, public_key) = self.key_pairs.get(address).unwrap();
        let signature = odra_types::casper_types::crypto::sign(message, secret_key, public_key)
            .to_bytes()
            .unwrap();
        Bytes::from(signature)
    }
}
