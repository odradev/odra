use odra_core::casper_types::{
    AddressableEntity, AddressableEntityHash, GenesisConfig, GenesisConfigBuilder, Package,
    PackageHash, ProtocolVersion
};
use odra_core::consts::*;
use odra_core::prelude::*;
use odra_core::OdraResult;
use std::cell::RefCell;
use std::env;
use std::path::PathBuf;

use casper_engine_test_support::{
    DeployItemBuilder, EntityWithNamedKeys, ExecuteRequestBuilder, LmdbWasmTestBuilder,
    WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNTS, DEFAULT_ACCOUNT_INITIAL_BALANCE,
    DEFAULT_AUCTION_DELAY, DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_EXEC_CONFIG,
    DEFAULT_GENESIS_CONFIG_HASH, DEFAULT_GENESIS_TIMESTAMP_MILLIS,
    DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS, DEFAULT_PAYMENT, DEFAULT_ROUND_SEIGNIORAGE_RATE,
    DEFAULT_SYSTEM_CONFIG, DEFAULT_UNBONDING_DELAY, DEFAULT_VALIDATOR_SLOTS, DEFAULT_WASM_CONFIG
};
use casper_event_standard::try_full_name_from_bytes;
use casper_execution_engine::{engine_state, execution};
use casper_storage::data_access_layer::{DataAccessLayer, GenesisRequest};
use odra_core::{casper_event_standard, DeployReport, GasReport};
use std::rc::Rc;

use odra_core::casper_types::account::{Account, AccountHash};
use odra_core::casper_types::addressable_entity::NamedKeys;
use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::contracts::{ContractHash, ContractPackageHash};
use odra_core::casper_types::{
    bytesrepr::FromBytes, CLTyped, GenesisAccount, PublicKey, RuntimeArgs, U512
};
use odra_core::casper_types::{
    runtime_args, ApiError, BlockTime, Contract, Key, Motes, SecretKey, StoredValue, URef
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

/// Casper virtual machine utilizing [LmdbWasmTestBuilder].
pub struct CasperVm {
    accounts: Vec<Address>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    active_account: Address,
    context: LmdbWasmTestBuilder,
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
        let named_keys = self
            .context
            .get_named_keys_by_account_hash(self.active_account_hash());

        let key: &Key = named_keys.get(name).unwrap();
        ContractPackageHash::from(key.into_package_hash().unwrap().value())
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

    /// Gets the current block time.
    pub fn block_time(&self) -> u64 {
        self.block_time
    }

    /// Gets the event at the specified index for the given contract address.
    ///
    /// The index may be negative, in which case it is interpreted as an offset from the end of the event list.
    ///
    /// Returns [EventError::IndexOutOfBounds] if the index is out of bounds.
    pub fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        let contract_package_hash = contract_address.as_contract_package_hash().unwrap();
        let contract_hash: ContractHash = self.get_contract_hash(contract_package_hash);

        let dictionary_seed_uref = self
            .package_named_key(*contract_package_hash, EVENTS)
            .ok_or(EventError::ContractDoesntSupportEvents)?;

        Ok(self.get_dict_value(*dictionary_seed_uref.as_uref().unwrap(), &index.to_string()))
        // TODO: Handle errors properly...
        // match self.context.query_dictionary_item(
        //     None,
        //     *dictionary_seed_uref.as_uref().unwrap(),
        //     &index.to_string()
        // ) {
        //     Ok(val) => {
        //         let bytes = val
        //             .as_cl_value()
        //             .unwrap()
        //             .clone()
        //             .into_t::<Bytes>()
        //             .unwrap();
        //         Ok(bytes)
        //     }
        //     Err(_) => Err(EventError::IndexOutOfBounds)
        // }
    }

    /// Gets the count of events for the given contract address.
    pub fn get_events_count(&self, contract_address: &Address) -> u32 {
        let contract_package_hash = contract_address
            .as_contract_package_hash()
            .expect("Events can only be queried for contracts");
        self.events_length(*contract_package_hash)
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
        let hash = address.as_package_hash().expect("Contract hash expected");

        let deploy_item = if use_proxy {
            let session_code =
                include_bytes!("../../resources/proxy_caller_with_return.wasm").to_vec();
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

            DeployItemBuilder::new()
                .with_standard_payment(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
                .with_authorization_keys(&[self.active_account_hash()])
                .with_address(self.active_account_hash())
                .with_session_bytes(session_code, args)
                .with_deploy_hash(self.next_hash())
                .build()
        } else {
            DeployItemBuilder::new()
                .with_standard_payment(runtime_args! { ARG_AMOUNT => *DEFAULT_PAYMENT})
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

        let execute_request = ExecuteRequestBuilder::from_deploy_item(&deploy_item)
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

        let result = self.deploy_contract(&wasm_path, &init_args);
        if let Some(error) = result {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            panic!("Revert: Contract deploy failed {:?}", odra_error);
        } else {
            let contract_package_hash =
                self.contract_package_hash_from_name(&package_hash_key_name);
            contract_package_hash.try_into().unwrap()
        }
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
            Address::Contract(contract_package_hash) => {
                self.get_contract_cspr_balance(contract_package_hash)
            }
        }
    }

    /// Transfers the specified amount of tokens to the given address.
    ///
    /// Results an OdraError if the transfer fails.
    pub fn transfer(&mut self, to: Address, amount: U512) -> OdraResult<()> {
        let deploy_item = DeployItemBuilder::new()
            .with_transfer_args(runtime_args! {
                "amount" => amount,
                "target" => to,
                "id" => Some(0u64),
            })
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(&deploy_item)
            .with_block_time(self.block_time)
            .build();
        self.context.exec(execute_request).commit();

        if let Some(error) = self.context.get_error() {
            let odra_error = parse_error(error);
            Err(odra_error)
        } else {
            Ok(())
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
    /// Keep in mind that this may be different from the cost of the transaction on the live network.
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
        let account: AddressableEntity = self
            .context
            .get_entity_by_account_hash(*account_hash)
            .unwrap();
        let purse = account.main_purse();
        self.context.get_purse_balance(purse)
    }

    fn get_contract_cspr_balance(&self, contract_package_hash: &ContractPackageHash) -> U512 {
        // TODO: Addressable entity has main purse inside it, is it the same as ours for contracts?
        let purse_key = self.package_named_key(*contract_package_hash, CONTRACT_MAIN_PURSE);
        match purse_key {
            None => U512::zero(),
            Some(purse_key) => {
                let purse_uref = purse_key.as_uref().unwrap_or_else(|| {
                    panic!(
                        "Contract doesn't have main purse uref under {} named key",
                        CONTRACT_MAIN_PURSE
                    )
                });
                self.context.get_purse_balance(*purse_uref)
            }
        }
    }

    fn get_contract_hash(&self, contract_package_hash: &ContractPackageHash) -> ContractHash {
        ContractHash::new(contract_package_hash.value())
    }

    fn genesis_accounts(
        key_pairs: &BTreeMap<Address, (SecretKey, PublicKey)>
    ) -> Vec<GenesisAccount> {
        let mut accounts = Vec::new();
        for (_, (_, public_key)) in key_pairs.iter() {
            accounts.push(GenesisAccount::account(
                public_key.clone(),
                Motes::new(DEFAULT_ACCOUNT_INITIAL_BALANCE),
                None
            ));
        }
        accounts
    }

    /// Creates a new genesis config.
    /// It is the same as the default one, but with the given genesis,
    /// so we will know their private keys.
    fn genesis_config(genesis_accounts: Vec<GenesisAccount>) -> GenesisConfig {
        GenesisConfigBuilder::default()
            .with_accounts(genesis_accounts)
            .with_wasm_config(*DEFAULT_WASM_CONFIG)
            .with_system_config(*DEFAULT_SYSTEM_CONFIG)
            .with_validator_slots(DEFAULT_VALIDATOR_SLOTS)
            .with_auction_delay(DEFAULT_AUCTION_DELAY)
            .with_locked_funds_period_millis(DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS)
            .with_round_seigniorage_rate(DEFAULT_ROUND_SEIGNIORAGE_RATE)
            .with_unbonding_delay(DEFAULT_UNBONDING_DELAY)
            .with_genesis_timestamp_millis(DEFAULT_GENESIS_TIMESTAMP_MILLIS)
            .build()
    }

    fn new_instance() -> Self {
        let key_pairs = generate_key_pairs(ACCOUNTS_NUMBER);
        let genesis_accounts = Self::genesis_accounts(&key_pairs);
        let accounts: Vec<Address> = key_pairs.keys().copied().collect();

        let genesis_config = Self::genesis_config(genesis_accounts);

        let run_genesis_request = GenesisRequest::new(
            DEFAULT_GENESIS_CONFIG_HASH,
            ProtocolVersion::V2_0_0,
            genesis_config,
            DEFAULT_CHAINSPEC_REGISTRY.clone()
        );

        let chainspec_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/chainspec.toml");
        let mut builder = LmdbWasmTestBuilder::new_temporary_with_chainspec(chainspec_path);

        builder.run_genesis(run_genesis_request).commit();

        Self {
            active_account: accounts[0],
            context: builder,
            accounts,
            block_time: 0u64,
            calls_counter: 0,
            error: None,
            attached_value: U512::zero(),
            gas_used: BTreeMap::new(),
            gas_report: GasReport::default(),
            key_pairs
        }
    }

    fn deploy_contract(
        &mut self,
        wasm_path: &str,
        args: &RuntimeArgs
    ) -> Option<engine_state::Error> {
        self.error = None;
        let session_code = PathBuf::from(wasm_path);
        let deploy_item = DeployItemBuilder::new()
            .with_standard_payment(runtime_args! {ARG_AMOUNT => *DEFAULT_PAYMENT})
            .with_authorization_keys(&[self.active_account_hash()])
            .with_address(self.active_account_hash())
            .with_session_code(session_code, args.clone())
            .with_deploy_hash(self.next_hash())
            .build();

        let execute_request = ExecuteRequestBuilder::from_deploy_item(&deploy_item)
            .with_block_time(self.block_time)
            .build();
        let result = self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_report.push(DeployReport::WasmDeploy {
            gas: self.last_call_contract_gas_cost(),
            file_name: wasm_path.to_string()
        });
        self.context.get_error()
    }
}

impl CasperVm {
    fn get_package(&self, contract_package_hash: ContractPackageHash) -> Package {
        let stored_value = self
            .context
            .query(None, Key::Package(contract_package_hash.value()), &[])
            .unwrap();

        match stored_value {
            StoredValue::Package(package) => package,
            _ => panic!("Expected Package")
        }
    }

    fn get_current_contract(
        &self,
        contract_package_hash: ContractPackageHash
    ) -> EntityWithNamedKeys {
        let package = self.get_package(contract_package_hash);
        let addressable_entity_hash = package
            .current_entity_hash()
            .expect("Package doesn't have current entity hash");
        self.context
            .get_entity_with_named_keys_by_entity_hash(addressable_entity_hash)
            .expect("Couldn't find entity by hash")
    }

    /// Gets current contract from contract package and
    /// returns its named keys.
    fn package_named_keys(&self, contract_package_hash: ContractPackageHash) -> NamedKeys {
        let package = self.get_package(contract_package_hash);
        let addressable_entity_hash = package
            .current_entity_hash()
            .expect("Package doesn't have current entity hash");
        let contract = self
            .context
            .get_entity_with_named_keys_by_entity_hash(addressable_entity_hash)
            .expect("Entity not found");
        contract.named_keys().clone()
    }

    fn package_named_key(
        &self,
        contract_package_hash: ContractPackageHash,
        name: &str
    ) -> Option<Key> {
        let keys = self.package_named_keys(contract_package_hash);
        let key = keys.get(name);
        key.copied()
    }

    fn events_length(&self, contract_package_hash: ContractPackageHash) -> u32 {
        let key = self.package_named_key(contract_package_hash, EVENTS_LENGTH);
        match key {
            None => 0,
            Some(key) => self.get_value(key)
        }
    }

    // TODO: Make this return Result
    fn get_value<T: CLTyped + FromBytes>(&self, key: Key) -> T {
        let value = self.context.query(None, key, &[]);
        value
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t::<T>()
            .unwrap()
    }

    // Make this return Result also
    fn get_dict_value<T: CLTyped + FromBytes>(&self, uref: URef, name: &str) -> T {
        let value = self.context.query_dictionary_item(None, uref, name);
        value
            .unwrap()
            .as_cl_value()
            .unwrap()
            .clone()
            .into_t::<T>()
            .unwrap()
    }

    fn panic_with_error(&self, error: OdraError, entrypoint: &str, package_hash: PackageHash) -> ! {
        panic!("Revert: {:?} - {:?}::{}", error, package_hash, entrypoint)
    }
}

fn parse_error(err: engine_state::Error) -> OdraError {
    if let engine_state::Error::Exec(exec_err) = err {
        match exec_err {
            execution::ExecError::Revert(ApiError::MissingArgument) => {
                OdraError::ExecutionError(ExecutionError::MissingArg)
            }
            execution::ExecError::Revert(ApiError::Mint(0)) => {
                OdraError::VmError(VmError::BalanceExceeded)
            }
            execution::ExecError::Revert(ApiError::User(code)) => match code {
                x if x == ExecutionError::UnwrapError.code() => {
                    OdraError::ExecutionError(ExecutionError::UnwrapError)
                }
                x if x == ExecutionError::AdditionOverflow.code() => {
                    OdraError::ExecutionError(ExecutionError::AdditionOverflow)
                }
                x if x == ExecutionError::SubtractionOverflow.code() => {
                    OdraError::ExecutionError(ExecutionError::SubtractionOverflow)
                }
                x if x == ExecutionError::NonPayable.code() => {
                    OdraError::ExecutionError(ExecutionError::NonPayable)
                }
                x if x == ExecutionError::TransferToContract.code() => {
                    OdraError::ExecutionError(ExecutionError::TransferToContract)
                }
                x if x == ExecutionError::ReentrantCall.code() => {
                    OdraError::ExecutionError(ExecutionError::ReentrantCall)
                }
                x if x == ExecutionError::ContractAlreadyInstalled.code() => {
                    OdraError::ExecutionError(ExecutionError::ContractAlreadyInstalled)
                }
                x if x == ExecutionError::UnknownConstructor.code() => {
                    OdraError::ExecutionError(ExecutionError::UnknownConstructor)
                }
                x if x == ExecutionError::NativeTransferError.code() => {
                    OdraError::ExecutionError(ExecutionError::NativeTransferError)
                }
                x if x == ExecutionError::IndexOutOfBounds.code() => {
                    OdraError::ExecutionError(ExecutionError::IndexOutOfBounds)
                }
                x if x == ExecutionError::ZeroAddress.code() => {
                    OdraError::ExecutionError(ExecutionError::ZeroAddress)
                }
                x if x == ExecutionError::AddressCreationFailed.code() => {
                    OdraError::ExecutionError(ExecutionError::AddressCreationFailed)
                }
                x if x == ExecutionError::EarlyEndOfStream.code() => {
                    OdraError::ExecutionError(ExecutionError::EarlyEndOfStream)
                }
                x if x == ExecutionError::Formatting.code() => {
                    OdraError::ExecutionError(ExecutionError::Formatting)
                }
                x if x == ExecutionError::LeftOverBytes.code() => {
                    OdraError::ExecutionError(ExecutionError::LeftOverBytes)
                }
                x if x == ExecutionError::OutOfMemory.code() => {
                    OdraError::ExecutionError(ExecutionError::OutOfMemory)
                }
                x if x == ExecutionError::NotRepresentable.code() => {
                    OdraError::ExecutionError(ExecutionError::NotRepresentable)
                }
                x if x == ExecutionError::ExceededRecursionDepth.code() => {
                    OdraError::ExecutionError(ExecutionError::ExceededRecursionDepth)
                }
                x if x == ExecutionError::KeyNotFound.code() => {
                    OdraError::ExecutionError(ExecutionError::KeyNotFound)
                }
                x if x == ExecutionError::CouldNotDeserializeSignature.code() => {
                    OdraError::ExecutionError(ExecutionError::CouldNotDeserializeSignature)
                }
                x if x == ExecutionError::TypeMismatch.code() => {
                    OdraError::ExecutionError(ExecutionError::TypeMismatch)
                }
                x if x == ExecutionError::CouldNotSignMessage.code() => {
                    OdraError::ExecutionError(ExecutionError::CouldNotSignMessage)
                }
                x if x == ExecutionError::EmptyDictionaryName.code() => {
                    OdraError::ExecutionError(ExecutionError::EmptyDictionaryName)
                }
                x if x == ExecutionError::MissingArg.code() => {
                    OdraError::ExecutionError(ExecutionError::MissingArg)
                }
                _ => OdraError::ExecutionError(ExecutionError::User(code))
            },
            execution::ExecError::InvalidContext => OdraError::VmError(VmError::InvalidContext),
            execution::ExecError::NoSuchMethod(name) => {
                OdraError::VmError(VmError::NoSuchMethod(name))
            }
            execution::ExecError::MissingArgument { name } => {
                OdraError::ExecutionError(ExecutionError::MissingArg)
            }
            _ => OdraError::VmError(VmError::Other(format!("Casper ExecError: {}", exec_err)))
        }
    } else if let engine_state::Error::InsufficientPayment = err {
        OdraError::VmError(VmError::BalanceExceeded)
    } else {
        OdraError::VmError(VmError::Other(format!("Casper EngineStateError: {}", err)))
    }
}
