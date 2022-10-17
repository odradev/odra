//! Implementation of [CasperTestEnv].

use std::{cell::RefCell, path::PathBuf};

use casper_engine_test_support::{
    DeployItemBuilder, ExecuteRequestBuilder, InMemoryWasmTestBuilder, ARG_AMOUNT,
    DEFAULT_ACCOUNT_INITIAL_BALANCE, DEFAULT_GENESIS_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
    DEFAULT_PAYMENT,
};
use casper_execution_engine::core::engine_state::{
    self, run_genesis_request::RunGenesisRequest, GenesisAccount,
};
pub use casper_execution_engine::core::execution::Error as CasperExecutionError;
use casper_types::{
    account::{Account, AccountHash},
    bytesrepr::{Bytes, FromBytes, ToBytes},
    runtime_args, ApiError, CLTyped, Contract, ContractHash, ContractPackageHash, Key, Motes,
    PublicKey, RuntimeArgs, SecretKey, URef, U512,
};
use odra::types::{event::EventError, EventData, ExecutionError, OdraError, VmError};
use odra_casper_shared::{casper_address::CasperAddress, consts};

thread_local! {
    /// Thread local instance of [CasperTestEnv].
    pub static ENV: RefCell<CasperTestEnv> = RefCell::new(CasperTestEnv::new());
}

/// Wrapper for InMemoryWasmTestBuilder.
pub struct CasperTestEnv {
    accounts: Vec<CasperAddress>,
    active_account: CasperAddress,
    context: InMemoryWasmTestBuilder,
    block_time: u64,
    calls_counter: u32,
    error: Option<OdraError>,
    attached_value: Option<U512>,
}

impl CasperTestEnv {
    /// Create a new instance with predefined accounts.
    pub fn new() -> Self {
        let mut genesis_config = DEFAULT_GENESIS_CONFIG.clone();
        let mut accounts: Vec<CasperAddress> = Vec::new();
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
                None,
            );
            genesis_config.ee_config_mut().push_account(account);

            accounts.push(account_addr.into());
        }
        let run_genesis_request = RunGenesisRequest::new(
            *DEFAULT_GENESIS_CONFIG_HASH,
            genesis_config.protocol_version(),
            genesis_config.take_ee_config(),
        );

        let mut builder = InMemoryWasmTestBuilder::default();
        builder.run_genesis(&run_genesis_request).commit();

        Self {
            active_account: accounts[0],
            context: builder,
            accounts,
            block_time: 0,
            calls_counter: 0,
            error: None,
            attached_value: None,
        }
    }

    /// Deploy WASM file with args.
    pub fn deploy_contract(&mut self, wasm_path: &str, args: RuntimeArgs) {
        let session_code = PathBuf::from(wasm_path);
        let deploy_item = DeployItemBuilder::new()
            .with_empty_payment_bytes(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args)
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit().expect_success();
        self.active_account = self.get_account(0);
    }

    /// Call contract.
    pub fn call_contract(
        &mut self,
        hash: ContractPackageHash,
        entry_point: &str,
        args: RuntimeArgs,
        has_return: bool,
    ) -> Option<Bytes> {
        self.error = None;

        let session_code = include_bytes!("../getter_proxy.wasm").to_vec();
        let args_bytes: Vec<u8> = args.to_bytes().unwrap();
        let args = runtime_args! {
            "contract_package_hash" => hash,
            "entry_point" => entry_point,
            "args" => Bytes::from(args_bytes),
            "has_return" => has_return,
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

        let active_account = self.active_account_hash();
        self.attached_value = None;

        if self.context.is_error() {
            self.error = Some(parse_error(self.context.get_error().unwrap()));
            None
        } else if has_return {
            Some(self.get_account_value(active_account, "result"))
        } else {
            None
        }
    }

    /// Set caller.
    pub fn set_caller(&mut self, account: CasperAddress) {
        self.active_account = account;
    }

    /// Get one of the predefined accounts.
    pub fn get_account(&self, n: usize) -> CasperAddress {
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
    pub fn get_account_value<T: FromBytes + CLTyped>(&self, hash: AccountHash, name: &str) -> T {
        self.context
            .query(None, Key::Account(hash), &[name.to_string()])
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t()
            .unwrap()
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
    pub fn get_event(&self, address: CasperAddress, index: i32) -> Result<EventData, EventError> {
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
                &[String::from(consts::EVENTS_LENGTH)],
            )
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t()
            .unwrap();

        let event_position = odra::utils::event_absolute_position(events_length as usize, index)?;

        match self.context.query_dictionary_item(
            None,
            dictionary_seed_uref,
            &event_position.to_string(),
        ) {
            Ok(val) => {
                let value: Bytes = val
                    .as_cl_value()
                    .unwrap()
                    .clone()
                    .into_t::<Bytes>()
                    .unwrap();
                Ok(value.inner_bytes().clone())
            }
            Err(_) => Err(EventError::IndexOutOfBounds),
        }
    }

    /// Increases the current value of block_time.
    pub fn advance_block_time_by(&mut self, seconds: u64) {
        self.block_time += seconds;
    }

    /// Sets the value that will be attached to the next contract call.
    pub fn attach_value(&mut self, amount: U512) {
        self.attached_value = Some(amount);
    }

    /// Returns the balance of the given account.
    ///
    /// The accepted value can be either an [CasperAddress::Account] or [CasperAddress::Contract].
    pub fn token_balance(&self, address: CasperAddress) -> U512 {
        match address {
            CasperAddress::Account(account_hash) => self.get_account_cspr_balance(account_hash),
            CasperAddress::Contract(contract_hash) => self.get_contract_cspr_balance(contract_hash),
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
            .unwrap_or_else(|| U512::zero())
    }

    fn get_account_cspr_balance(&self, account_hash: AccountHash) -> U512 {
        let account: Account = self.context.get_account(account_hash).unwrap();
        let purse = account.main_purse();
        self.context.get_purse_balance(purse)
    }

    fn active_account_hash(&self) -> AccountHash {
        *self.active_account.as_account_hash().unwrap()
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
            _ => OdraError::VmError(VmError::Other(format!("Casper ExecError: {}", exec_err))),
        }
    } else {
        OdraError::VmError(VmError::Other(format!("Casper EngineStateError: {}", err)))
    }
}
