use odra_core::consts::*;
use odra_core::prelude::*;
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_GENESIS_CONFIG,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_PAYMENT
};
use casper_event_standard::try_full_name_from_bytes;
use odra_core::{casper_event_standard, DeployReport, GasReport};
use std::rc::Rc;

use casper_execution_engine::core::engine_state::{self, GenesisAccount, RunGenesisRequest};
use odra_core::casper_types::account::{Account, AccountHash};
use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::{bytesrepr::FromBytes, CLTyped, PublicKey, RuntimeArgs, U512};
use odra_core::casper_types::{
    runtime_args, ApiError, BlockTime, Contract, ContractHash, ContractPackageHash, Key, Motes,
    SecretKey, StoredValue, URef
};
use odra_core::consts;
use odra_core::consts::*;
use odra_core::crypto::generate_key_pairs;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::EventError;
use odra_core::{
    host::{HostContext, HostEnv},
    CallDef, ContractEnv
};
use odra_core::{Address, ExecutionError, OdraError, VmError};

/// Casper virtual machine utilizing [InMemoryWasmTestBuilder].
pub struct CasperVm {
    accounts: Vec<Address>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    active_account: Address,
    context: InMemoryWasmTestBuilder,
    block_time: u64,
    calls_counter: u32,
    error: Option<OdraError>,
    attached_value: U512,
    gas_used: BTreeMap<AccountHash, U512>,
    gas_report: GasReport
}

impl CasperVm {
    /// Creates a new instance with predefined accounts.
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
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

    /// Updates the active account (caller) address.
    pub fn set_caller(&mut self, caller: Address) {
        self.active_account = caller;
    }

    /// Gets the active account (caller) address.
    pub fn get_caller(&self) -> Address {
        self.active_account
    }

    /// Gets the account address at the specified index.
    pub fn get_account(&self, index: usize) -> Address {
        self.accounts[index]
    }

    /// Advances the block time by the specified time difference.
    pub fn advance_block_time(&mut self, time_diff: u64) {
        self.block_time += time_diff
    }

    /// Gets the event at the specified index for the given contract address.
    ///
    /// The index may be negative, in which case it is interpreted as an offset from the end of the event list.
    ///
    /// Returns [EventError::IndexOutOfBounds] if the index is out of bounds.
    pub fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        let contract_package_hash = contract_address.as_contract_package_hash().unwrap();
        let contract_hash: ContractHash = self.get_contract_package_hash(contract_package_hash);

        let dictionary_seed_uref: URef = *self
            .context
            .get_contract(contract_hash)
            .unwrap()
            .named_keys()
            .get(consts::EVENTS)
            .unwrap()
            .as_uref()
            .unwrap();

        match self
            .context
            .query_dictionary_item(None, dictionary_seed_uref, &index.to_string())
        {
            Ok(val) => {
                let bytes = val
                    .as_cl_value()
                    .unwrap()
                    .clone()
                    .into_t::<Bytes>()
                    .unwrap();
                Ok(bytes)
            }
            Err(_) => Err(EventError::IndexOutOfBounds)
        }
    }

    /// Gets the count of events for the given contract address.
    pub fn get_events_count(&self, contract_address: &Address) -> u32 {
        let contract_package_hash = contract_address.as_contract_package_hash().unwrap();
        let contract_hash: ContractHash = self.get_contract_package_hash(contract_package_hash);

        self.events_length(&contract_hash)
    }

    /// Attaches a value to the next call.
    pub fn attach_value(&mut self, amount: U512) {
        self.attached_value = amount;
    }

    /// Calls a contract with the specified address, call definition, and proxy usage flag.
    ///
    /// If the proxy usage flag is set to true, then the contract will be called via a proxy caller.
    pub fn call_contract(
        &mut self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> Bytes {
        self.error = None;
        let hash = *address
            .as_contract_package_hash()
            .expect("Contract hash expected");

        let deploy_item = if use_proxy {
            let session_code =
                include_bytes!("../../resources/proxy_caller_with_return.wasm").to_vec();
            let args_bytes: Vec<u8> = call_def
                .args()
                .to_bytes()
                .expect("Should serialize to bytes");
            let entry_point = call_def.entry_point().to_string();
            let args = runtime_args! {
                CONTRACT_PACKAGE_HASH_ARG => hash,
                ENTRY_POINT_ARG => entry_point,
                ARGS_ARG => Bytes::from(args_bytes),
                ATTACHED_VALUE_ARG => call_def.amount(),
                AMOUNT_ARG => call_def.amount(),
            };

            DeployItemBuilder::new()
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_authorization_keys(&[self.active_account_hash()])
                .with_address(self.active_account_hash())
                .with_session_bytes(session_code, args)
                .with_deploy_hash(self.next_hash())
                .build()
        } else {
            DeployItemBuilder::new()
                .with_empty_payment_bytes(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_authorization_keys(&[self.active_account_hash()])
                .with_address(self.active_account_hash())
                .with_stored_versioned_contract_by_hash(
                    hash.value(),
                    None,
                    call_def.entry_point(),
                    call_def.args().clone()
                )
                .with_deploy_hash(self.next_hash())
                .build()
        };

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_report.push(DeployReport::ContractCall {
            gas: self.last_call_contract_gas_cost(),
            contract_address: *address,
            call_def: call_def.clone()
        });

        self.attached_value = U512::zero();
        if let Some(error) = self.context.get_error() {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            self.panic_with_error(odra_error, call_def.entry_point(), hash);
        } else {
            self.get_active_account_result()
        }
    }

    /// Creates a new contract with the specified name, initialization arguments, and entry points caller.
    pub fn new_contract(
        &mut self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let wasm_path = format!("{}.wasm", name);
        let package_hash_key_name: String = init_args
            .get(PACKAGE_HASH_KEY_NAME_ARG)
            .unwrap()
            .clone()
            .into_t()
            .unwrap();

        self.deploy_contract(&wasm_path, &init_args);
        let contract_package_hash = self.contract_package_hash_from_name(&package_hash_key_name);
        contract_package_hash.try_into().unwrap()
    }

    /// Create a new instance with predefined accounts.
    pub fn active_account_hash(&self) -> AccountHash {
        *self.active_account.as_account_hash().unwrap()
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

    /// Returns the cost of the last deploy.
    /// Keep in mind that this may be different from the cost of the deploy on the live network.
    /// This is NOT the amount of gas charged - see [last_call_contract_gas_used()](Self::last_call_contract_gas_used).
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
    pub fn gas_report(&self) -> &GasReport {
        &self.gas_report
    }

    /// Returns the public key that corresponds to the given Account Address.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        let (_, public_key) = self.key_pairs.get(address).unwrap();
        public_key.clone()
    }

    /// Cryptographically signs a message as a given account.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        let (secret_key, public_key) = self.key_pairs.get(address).unwrap();
        let signature = odra_core::casper_types::crypto::sign(message, secret_key, public_key)
            .to_bytes()
            .unwrap();
        Bytes::from(signature)
    }

    /// Gets the gas cost of the last contract call.
    pub fn last_call_gas_cost(&self) -> u64 {
        self.last_call_contract_gas_cost().as_u64()
    }

    /// Gets the error, if any, encountered during execution.
    pub fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    fn get_active_account_result(&self) -> Bytes {
        let active_account = self.active_account_hash();
        let bytes: Bytes = self
            .get_account_value(active_account, RESULT_KEY)
            .unwrap_or_default();
        bytes
    }

    fn collect_gas(&mut self) {
        *self
            .gas_used
            .entry(*self.active_account.as_account_hash().unwrap())
            .or_insert_with(U512::zero) += *DEFAULT_PAYMENT;
    }

    fn next_hash(&mut self) -> [u8; 32] {
        let seed = self.calls_counter;
        self.calls_counter += 1;
        let mut hash = [0u8; 32];
        hash[0] = seed as u8;
        hash[1] = (seed >> 8) as u8;
        hash
    }

    fn get_account_cspr_balance(&self, account_hash: &AccountHash) -> U512 {
        let account: Account = self.context.get_account(*account_hash).unwrap();
        let purse = account.main_purse();
        let gas_used = self
            .gas_used
            .get(account_hash)
            .copied()
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
            .get_contract_package(*contract_hash)
            .unwrap()
            .current_contract_hash()
            .unwrap()
    }

    fn new_instance() -> Self {
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        let mut accounts: Vec<Address> = Vec::new();
        let key_pairs = generate_key_pairs(20);
        key_pairs
            .iter()
            .for_each(|(address, (secret_key, public_key))| {
                accounts.push(*address);
                let account = GenesisAccount::account(
                    public_key.clone(),
                    Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
                    None
                );
                genesis_config.ee_config_mut().push_account(account);
            });

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
            gas_report: Vec::new(),
            key_pairs
        }
    }

    fn deploy_contract(&mut self, wasm_path: &str, args: &RuntimeArgs) {
        self.error = None;
        let session_code = PathBuf::from(wasm_path);

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args.clone())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit().expect_success();
        self.collect_gas();
        self.gas_report.push(DeployReport::WasmDeploy {
            gas: self.last_call_contract_gas_cost(),
            file_name: wasm_path.to_string()
        });
    }
}

impl CasperVm {
    fn events_length(&self, contract_hash: &ContractHash) -> u32 {
        self.context
            .query(
                None,
                Key::Hash(contract_hash.value()),
                &[String::from(consts::EVENTS_LENGTH)]
            )
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t()
            .unwrap()
    }

    fn panic_with_error(
        &self,
        error: OdraError,
        entrypoint: &str,
        contract_package_hash: ContractPackageHash
    ) -> ! {
        panic!(
            "Revert: {:?} - {:?}::{}",
            error, contract_package_hash, entrypoint
        )
    }
}

fn parse_error(err: engine_state::Error) -> OdraError {
    if let engine_state::Error::Exec(exec_err) = err {
        match exec_err {
            engine_state::ExecError::Revert(ApiError::MissingArgument) => {
                OdraError::VmError(VmError::MissingArg)
            }
            engine_state::ExecError::Revert(ApiError::Mint(0)) => {
                OdraError::VmError(VmError::BalanceExceeded)
            }
            engine_state::ExecError::Revert(ApiError::User(code)) => {
                if code == ExecutionError::NonPayable.code() {
                    OdraError::ExecutionError(ExecutionError::NonPayable)
                } else if code == ExecutionError::ReentrantCall.code() {
                    OdraError::ExecutionError(ExecutionError::ReentrantCall)
                } else {
                    OdraError::ExecutionError(ExecutionError::User(code))
                }
            }
            engine_state::ExecError::InvalidContext => OdraError::VmError(VmError::InvalidContext),
            engine_state::ExecError::NoSuchMethod(name) => {
                OdraError::VmError(VmError::NoSuchMethod(name))
            }
            engine_state::ExecError::MissingArgument { name } => {
                OdraError::VmError(VmError::MissingArg)
            }
            _ => OdraError::VmError(VmError::Other(format!("Casper ExecError: {}", exec_err)))
        }
    } else {
        OdraError::VmError(VmError::Other(format!("Casper EngineStateError: {}", err)))
    }
}
