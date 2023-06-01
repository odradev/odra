//! Implementation of [CasperTestEnv].

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    cell::RefCell,
    collections::HashMap,
    env,
    path::PathBuf
};

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
    DEFAULT_PAYMENT
};
use casper_execution_engine::core::engine_state::{
    self, run_genesis_request::RunGenesisRequest, GenesisAccount
};
pub use casper_execution_engine::core::execution::Error as CasperExecutionError;
use casper_types::{
    account::{Account, AccountHash},
    bytesrepr::{Bytes, FromBytes, ToBytes},
    runtime_args, ApiError, Contract, ContractHash, ContractPackageHash, Key, Motes, PublicKey,
    RuntimeArgs, SecretKey, StoredValue, URef, U512
};
use odra_casper_shared::consts;
use odra_casper_types::{Address, BlockTime, CallArgs, OdraType};
use odra_types::{
    event::{EventError, OdraEvent},
    ExecutionError, OdraError, VmError
};

use crate::debug;

thread_local! {
    /// Thread local instance of [CasperTestEnv].
    pub static ENV: RefCell<CasperTestEnv> = RefCell::new(CasperTestEnv::new());
}

const ODRA_WASM_PATH_ENV_KEY: &str = "ODRA_WASM_PATH";

/// Wrapper for InMemoryWasmTestBuilder.
pub struct CasperTestEnv {
    accounts: Vec<Address>,
    active_account: Address,
    context: InMemoryWasmTestBuilder,
    block_time: BlockTime,
    calls_counter: u32,
    error: Option<OdraError>,
    attached_value: Option<U512>,
    gas_used: HashMap<AccountHash, U512>
}

impl CasperTestEnv {
    /// Create a new instance with predefined accounts.
    pub fn new() -> Self {
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        let mut accounts: Vec<Address> = Vec::new();
        for i in 0..20 {
            // Create keypair.
            let secret_key = SecretKey::ed25519_from_bytes([i; 32]).unwrap();
            let public_key = PublicKey::from(&secret_key);

            // Create an AccountHash from a public key.
            let account_addr = AccountHash::from(&public_key);

            // Create a GenesisAccount.
            let account = GenesisAccount::account(
                public_key,
                Motes::new(U512::from(DEFAULT_ACCOUNT_INITIAL_BALANCE)),
                None
            );
            genesis_config.ee_config_mut().push_account(account);

            accounts.push(account_addr.try_into().unwrap());
        }
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config()
        );

        let chainspec_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/chainspec.toml");

        let mut builder = InMemoryWasmTestBuilder::new_with_chainspec(chainspec_path, None);
        builder.run_genesis(&run_genesis_request).commit();

        Self {
            active_account: accounts[0],
            context: builder,
            accounts,
            block_time: 0,
            calls_counter: 0,
            error: None,
            attached_value: None,
            gas_used: HashMap::new()
        }
    }

    /// Deploy WASM file with args.
    pub fn deploy_contract(&mut self, wasm_path: &str, args: &CallArgs) {
        self.error = None;
        let mut session_code = PathBuf::from(wasm_path);
        if let Ok(path) = env::var(ODRA_WASM_PATH_ENV_KEY) {
            let mut path = PathBuf::from(path);
            path.push(wasm_path);
            if path.exists() {
                session_code = path;
            } else {
                panic!("WASM file not found: {:?}", path);
            }
        }

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args.as_casper_runtime_args().clone())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit().expect_success();
        self.collect_gas();
    }

    /// Call contract.
    pub fn call_contract<T: OdraType>(
        &mut self,
        hash: ContractPackageHash,
        entry_point: &str,
        args: &CallArgs
    ) -> T {
        self.error = None;

        let session_code = include_bytes!("../getter_proxy.wasm").to_vec();
        let args_bytes: Vec<u8> = args.to_bytes().unwrap();
        let args = runtime_args! {
            "contract_package_hash" => hash,
            "entry_point" => entry_point,
            "args" => Bytes::from(args_bytes),
            "attached_value" => self.attached_value,
            "amount" => self.attached_value.unwrap_or_default(),
        };

        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_bytes(session_code, args)
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();

        self.attached_value = None;
        if let Some(error) = self.context.get_error() {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            self.panic_with_error(odra_error, entry_point, hash);
        } else {
            self.get_active_account_result()
        }
    }

    fn panic_with_error(
        &self,
        error: OdraError,
        entrypoint: &str,
        contract_package_hash: ContractPackageHash
    ) -> ! {
        std::panic::set_hook(Box::new(|info| {
            let backtrace = Backtrace::capture();
            if matches!(backtrace.status(), BacktraceStatus::Captured) {
                debug::print_first_n_frames(&backtrace, 30);
            }
            debug::print_panic_error(info);
        }));

        panic!(
            "{}",
            debug::format_panic_message(&error, entrypoint, contract_package_hash)
        )
    }

    /// Set caller.
    pub fn set_caller(&mut self, account: Address) {
        self.active_account = account;
    }

    /// Get one of the predefined accounts.
    pub fn get_account(&self, n: usize) -> Address {
        *self.accounts.get(n).unwrap()
    }

    fn next_hash(&mut self) -> [u8; 32] {
        let seed = self.calls_counter;
        self.calls_counter += 1;
        let mut hash = [0u8; 32];
        hash[0] = seed as u8;
        hash[1] = (seed >> 8) as u8;
        hash
    }

    /// Read a value from Account's named keys.
    pub fn get_account_value<T: OdraType>(
        &self,
        hash: AccountHash,
        name: &str
    ) -> Result<T, String> {
        let result: Result<StoredValue, String> =
            self.context
                .query(None, Key::Account(hash), &[name.to_string()]);

        result.map(|value| value.as_cl_value().unwrap().clone().into_t().unwrap())
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

    /// Returns possible error.
    pub fn get_error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    /// Returns an event from the given contract.
    pub fn get_event<T: OdraType + OdraEvent>(
        &self,
        address: Address,
        index: i32
    ) -> Result<T, EventError> {
        let address = address.as_contract_package_hash().unwrap();

        let contract_hash: ContractHash = self.get_contract_package_hash(*address);

        let dictionary_seed_uref: URef = *self
            .context
            .get_contract(contract_hash)
            .unwrap()
            .named_keys()
            .get(consts::EVENTS)
            .unwrap()
            .as_uref()
            .unwrap();

        let events_length: u32 = self
            .context
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
            .unwrap();

        let event_position = odra_utils::event_absolute_position(events_length as usize, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        match self.context.query_dictionary_item(
            None,
            dictionary_seed_uref,
            &event_position.to_string()
        ) {
            Ok(val) => {
                let bytes = val
                    .as_cl_value()
                    .unwrap()
                    .clone()
                    .into_t::<Bytes>()
                    .unwrap();
                let event_type = CasperTestEnv::get_event_name(bytes.as_slice())?;
                if event_type != format!("event_{}", T::name()) {
                    return Err(EventError::UnexpectedType(event_type));
                }
                let value: T = T::from_bytes(bytes.as_slice()).unwrap().0;
                Ok(value)
            }
            Err(_) => Err(EventError::IndexOutOfBounds)
        }
    }

    /// Increases the current value of block_time.
    pub fn advance_block_time_by(&mut self, milliseconds: BlockTime) {
        self.block_time += milliseconds;
    }

    /// Sets the value that will be attached to the next contract call.
    pub fn attach_value(&mut self, amount: U512) {
        self.attached_value = Some(amount);
    }

    /// Returns the balance of the given account.
    ///
    /// The accepted value can be either an [Address::Account] or [Address::Contract].
    pub fn token_balance(&self, address: Address) -> U512 {
        match address {
            Address::Account(account_hash) => self.get_account_cspr_balance(account_hash),
            Address::Contract(contract_hash) => self.get_contract_cspr_balance(contract_hash)
        }
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
}

impl CasperTestEnv {
    fn get_contract_package_hash(&self, contract_hash: ContractPackageHash) -> ContractHash {
        self.context
            .get_contract_package(contract_hash)
            .unwrap()
            .current_contract_hash()
            .unwrap()
    }

    fn get_contract_cspr_balance(&self, contract_hash: ContractPackageHash) -> U512 {
        let contract_hash: ContractHash = self.get_contract_package_hash(contract_hash);
        let contract: Contract = self.context.get_contract(contract_hash).unwrap();
        contract
            .named_keys()
            .get(consts::CONTRACT_MAIN_PURSE)
            .and_then(|key| key.as_uref())
            .map(|purse| self.context.get_purse_balance(*purse))
            .unwrap_or_else(U512::zero)
    }

    fn get_account_cspr_balance(&self, account_hash: AccountHash) -> U512 {
        let account: Account = self.context.get_account(account_hash).unwrap();
        let purse = account.main_purse();
        self.context.get_purse_balance(purse)
    }

    fn get_event_name(bytes: &[u8]) -> Result<String, EventError> {
        let (event_name, _): (String, _) =
            FromBytes::from_bytes(bytes).map_err(|_| EventError::Formatting)?;
        Ok(event_name)
    }

    fn active_account_hash(&self) -> AccountHash {
        *self.active_account.as_account_hash().unwrap()
    }

    fn get_active_account_result<T: OdraType>(&self) -> T {
        let active_account = self.active_account_hash();
        let bytes: casper_types::bytesrepr::Bytes = self
            .get_account_value(active_account, "result")
            .unwrap_or_default();
        T::from_bytes(bytes.inner_bytes()).unwrap().0
    }

    fn collect_gas(&mut self) {
        *self
            .gas_used
            .entry(*self.active_account.as_account_hash().unwrap())
            .or_insert_with(U512::zero) += *DEFAULT_PAYMENT;
    }
}

impl Default for CasperTestEnv {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_error(err: engine_state::Error) -> OdraError {
    if let engine_state::Error::Exec(exec_err) = err {
        match exec_err {
            CasperExecutionError::Revert(ApiError::MissingArgument) => {
                OdraError::VmError(VmError::MissingArg)
            }
            CasperExecutionError::Revert(ApiError::User(id)) => {
                if id == ExecutionError::non_payable().code() {
                    OdraError::ExecutionError(ExecutionError::non_payable())
                } else if id == ExecutionError::reentrant_call().code() {
                    OdraError::ExecutionError(ExecutionError::reentrant_call())
                } else {
                    OdraError::ExecutionError(ExecutionError::new(id, ""))
                }
            }
            CasperExecutionError::InvalidContext => OdraError::VmError(VmError::InvalidContext),
            CasperExecutionError::MissingArgument { name: _ } => {
                OdraError::VmError(VmError::MissingArg)
            }
            CasperExecutionError::NoSuchMethod(name) => {
                OdraError::VmError(VmError::NoSuchMethod(name))
            }
            CasperExecutionError::Revert(ApiError::Mint(0)) => {
                OdraError::VmError(VmError::BalanceExceeded)
            }
            _ => OdraError::VmError(VmError::Other(format!("Casper ExecError: {}", exec_err)))
        }
    } else {
        OdraError::VmError(VmError::Other(format!("Casper EngineStateError: {}", err)))
    }
}
