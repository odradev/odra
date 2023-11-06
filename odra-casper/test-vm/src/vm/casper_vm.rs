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
use odra_types::casper_types::account::{Account, AccountHash};
use odra_types::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_types::casper_types::{
    runtime_args, BlockTime, Contract, ContractHash, ContractPackageHash, Key, Motes, SecretKey,
    StoredValue
};
use odra_types::{Address, PublicKey, U512};
use odra_types::{CLTyped, FromBytes, OdraError, RuntimeArgs};

pub struct CasperVm {
    pub accounts: Vec<Address>,
    pub key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    pub active_account: Address,
    pub context: InMemoryWasmTestBuilder,
    pub block_time: u64,
    pub calls_counter: u32,
    pub error: Option<OdraError>,
    pub attached_value: U512,
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
            block_time: 0u64,
            calls_counter: 0,
            error: None,
            attached_value: U512::zero(),
            gas_used: BTreeMap::new(),
            gas_cost: Vec::new(),
            key_pairs
        }
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

    pub fn advance_block_time(&mut self, time_diff: u64) {
        self.block_time += time_diff
    }

    pub fn get_event(
        &self,
        contract_address: Address,
        index: i32
    ) -> Option<odra_types::EventData> {
        todo!()
    }

    pub fn attach_value(&mut self, amount: U512) {
        self.attached_value = amount;
    }

    pub fn call_contract(
        &mut self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> Bytes {
        self.error = None;
        // TODO: handle unwrap
        let hash = *address.as_contract_package_hash().unwrap();

        let deploy_item = if use_proxy {
            let session_code =
                include_bytes!("../../resources/proxy_caller_with_return.wasm").to_vec();
            let args_bytes: Vec<u8> = call_def.args.to_bytes().unwrap();
            let entry_point = call_def.entry_point.clone();
            let args = runtime_args! {
                CONTRACT_PACKAGE_HASH_ARG => hash,
                ENTRY_POINT_ARG => entry_point,
                ARGS_ARG => Bytes::from(args_bytes),
                ATTACHED_VALUE_ARG => call_def.amount,
                AMOUNT_ARG => call_def.amount,
            };

            let deploy_item = DeployItemBuilder::new()
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_authorization_keys(&[self.active_account_hash()])
                .with_address(self.active_account_hash())
                .with_session_bytes(session_code, args)
                .with_deploy_hash(self.next_hash())
                .build();
            deploy_item
        } else {
            let deploy_item = DeployItemBuilder::new()
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_authorization_keys(&[self.active_account_hash()])
                .with_address(self.active_account_hash())
                .with_stored_versioned_contract_by_hash(
                    hash.value(),
                    None,
                    &call_def.entry_point,
                    call_def.args.clone()
                )
                .with_deploy_hash(self.next_hash())
                .build();
            deploy_item

            // let session_code = include_bytes!("../../resources/proxy_caller_with_return.wasm").to_vec();
            // let args_bytes: Vec<u8> = call_def.args.to_bytes().unwrap();
            // let entry_point = call_def.entry_point.clone();
            // let args = runtime_args! {
            //     CONTRACT_PACKAGE_HASH_ARG => hash,
            //     ENTRY_POINT_ARG => entry_point,
            //     ARGS_ARG => Bytes::from(args_bytes),
            //     ATTACHED_VALUE_ARG => call_def.amount,
            //     AMOUNT_ARG => call_def.amount,
            // };

            // let deploy_item = DeployItemBuilder::new()
            //     .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
            //     .with_authorization_keys(&[self.active_account_hash()])
            //     .with_address(self.active_account_hash())
            //     .with_session_bytes(session_code, args)
            //     .with_deploy_hash(self.next_hash())
            //     .build();
            // deploy_item
        };

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(u64::from(self.block_time))
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_cost.push((
            format!("call_entrypoint {}", call_def.entry_point),
            self.last_call_contract_gas_cost()
        ));

        self.attached_value = U512::zero();
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

    /// Returns the balance of the given address.
    ///
    /// The accepted value can be either an [Address::Account] or [Address::Contract].
    pub fn balance_of(&self, address: &Address) -> U512 {
        match address {
            Address::Account(account_hash) => self.get_account_cspr_balance(account_hash),
            Address::Contract(contract_hash) => self.get_contract_cspr_balance(contract_hash)
        }
    }

    fn get_account_cspr_balance(&self, account_hash: &AccountHash) -> U512 {
        let account: Account = self.context.get_account(account_hash.clone()).unwrap();
        let purse = account.main_purse();
        let gas_used = self
            .gas_used
            .get(account_hash)
            .map(|x| *x)
            .unwrap_or(U512::zero());
        self.context.get_purse_balance(purse) + gas_used
    }

    fn get_contract_cspr_balance(&self, contract_hash: &ContractPackageHash) -> U512 {
        let contract_hash: ContractHash = self.get_contract_package_hash(contract_hash);
        let contract: Contract = self.context.get_contract(contract_hash).unwrap();
        contract
            .named_keys()
            .get(consts::CONTRACT_MAIN_PURSE)
            .and_then(|key| key.as_uref())
            .map(|purse| self.context.get_purse_balance(*purse))
            .unwrap_or_else(U512::zero)
    }

    fn get_contract_package_hash(&self, contract_hash: &ContractPackageHash) -> ContractHash {
        self.context
            .get_contract_package(contract_hash.clone())
            .unwrap()
            .current_contract_hash()
            .unwrap()
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

    pub fn print_gas_report(&self) {
        println!("Gas report:");
        for (name, cost) in self.gas_report() {
            println!("{}: {}", name, cost);
        }
    }
}
