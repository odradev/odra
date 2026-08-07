use casper_engine_test_support::genesis_config_builder::GenesisConfigBuilder;
use odra_core::casper_types::system::auction::{
    BidAddr, BidKind, DelegationRate, DelegatorKind, ARG_DELEGATION_RATE, ARG_ENTRY_POINT,
    ARG_EVICTED_VALIDATORS, ARG_MINIMUM_DELEGATION_AMOUNT, ARG_PUBLIC_KEY, ARG_REWARDS_MAP,
    BLOCK_REWARD, METHOD_ADD_BID, METHOD_DISTRIBUTE, METHOD_RUN_AUCTION, METHOD_WITHDRAW_BID
};
use odra_core::casper_types::{
    AddressableEntity, AddressableEntityHash, EntityAddr, GenesisConfig, GenesisValidator,
    HashAddr, NamedKeys, Package, PackageHash, ProtocolVersion
};
use odra_core::consts::*;
use odra_core::prelude::*;
use std::cell::RefCell;
use std::env;
use std::hash::Hash;
use std::path::PathBuf;

use casper_engine_test_support::{
    ChainspecConfig, DeployItemBuilder, EntityWithNamedKeys, ExecuteRequestBuilder,
    TransferRequestBuilder, UpgradeRequestBuilder, CHAINSPEC_SYMLINK,
    LmdbWasmTestBuilder, WasmTestBuilder, ARG_AMOUNT, DEFAULT_ACCOUNTS, DEFAULT_AUCTION_DELAY,
    DEFAULT_CHAINSPEC_REGISTRY, DEFAULT_EXEC_CONFIG, DEFAULT_GENESIS_CONFIG_HASH,
    DEFAULT_GENESIS_TIMESTAMP_MILLIS, DEFAULT_LOCKED_FUNDS_PERIOD_MILLIS, DEFAULT_PAYMENT,
    DEFAULT_PROTOCOL_VERSION, DEFAULT_ROUND_SEIGNIORAGE_RATE, DEFAULT_SYSTEM_CONFIG,
    DEFAULT_UNBONDING_DELAY, DEFAULT_VALIDATOR_SLOTS, DEFAULT_WASM_CONFIG, SYSTEM_ADDR
};
use casper_event_standard::try_full_name_from_bytes;
use casper_execution_engine::{engine_state, execution};
use casper_storage::data_access_layer::{
    DataAccessLayer, GenesisRequest, RewardItem, StepRequest, StepResult
};
use odra_core::{casper_event_standard, DeployReport, GasReport};
use std::rc::Rc;

use odra_core::casper_event_standard::Schemas;
use odra_core::casper_types::account::{Account, AccountHash};
use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
use odra_core::casper_types::contract_messages::MessagePayload;
use odra_core::casper_types::contracts::{ContractHash, ContractPackageHash};
use odra_core::casper_types::{
    bytesrepr::FromBytes, CLTyped, GenesisAccount, PublicKey, RuntimeArgs, U512
};
use odra_core::casper_types::{
    runtime_args, ApiError, BlockTime, Contract, HoldBalanceHandling, Key, Motes, SecretKey,
    StoredValue, URef
};
use odra_core::consts;
use odra_core::consts::*;
use odra_core::crypto::generate_key_pairs;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::prelude::*;
use odra_core::EventError;
use odra_core::VmError;
use odra_core::CASPER_ERROR_GENERIC_NAME;
use odra_core::{
    host::{HostContext, HostEnv},
    CallDef, ContractEnv
};

/// Casper virtual machine utilizing [LmdbWasmTestBuilder].
pub struct CasperVm {
    accounts: Vec<Address>,
    validators: Vec<GenesisAccount>,
    removed_validators: Vec<PublicKey>,
    key_pairs: BTreeMap<Address, (SecretKey, PublicKey)>,
    messages: BTreeMap<EntityAddr, Vec<MessagePayload>>,
    active_account: Address,
    context: LmdbWasmTestBuilder,
    data_dir: Rc<tempfile::TempDir>,
    chainspec: ChainspecConfig,
    protocol_version: ProtocolVersion,
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

    /// Read a PackageHash of a given name, from the active account.
    pub fn package_hash_from_name(&self, name: &str) -> PackageHash {
        let named_keys = self
            .context
            .get_named_keys_by_account_hash(self.active_account_hash());

        let key: &Key = named_keys.get(name).unwrap();
        PackageHash::from(key.into_package_hash().unwrap().value())
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

    /// Gets the validator public key.
    pub fn get_validator(&self, index: usize) -> PublicKey {
        self.validators
            .get(index)
            .unwrap_or_else(|| panic!("Not enough validators: {}", index))
            .public_key()
    }

    /// Advances the block time by the specified time difference in milliseconds.
    pub fn advance_block_time(&mut self, time_diff_millis: u64) {
        self.block_time += time_diff_millis
    }

    /// Advances the block time by the specified time difference in milliseconds
    /// and processes auctions giving the rewards to the validators.
    pub fn advance_with_auctions(&mut self, time_diff_millis: u64) {
        let time_between_auctions = self.auction_delay();
        // Calculate how many auctions we can run based on time_diff
        let num_auctions = time_diff_millis / time_between_auctions;

        // Run auctions and distribute rewards one at a time
        for _ in 0..num_auctions {
            let mut step_request_builder = self
                .context
                .step_request_builder()
                .with_protocol_version(self.protocol_version);
            // distribute rewards to all validators
            let mut rewards = BTreeMap::new();
            for validator in &self.validators {
                if self.removed_validators.contains(&validator.public_key()) {
                    continue;
                }
                rewards.insert(validator.public_key(), vec![U512::from(BLOCK_REWARD)]);
                let reward_item = RewardItem::new(validator.public_key(), BLOCK_REWARD);
                step_request_builder = step_request_builder.with_reward_item(reward_item);
            }

            let step_request = step_request_builder.build();
            self.context.step(step_request);
            self.advance_era();
            self.advance_block_time(time_between_auctions);
            self.context
                .distribute(None, self.protocol_version, rewards, self.block_time);
        }

        // Run remaining auctions with the leftover time
        let remaining_time = time_diff_millis % time_between_auctions;
        self.advance_block_time(remaining_time);
    }

    /// Advances the chain by a single era, running the auction.
    /// Unlike [LmdbWasmTestBuilder::advance_era] it carries the VM's current
    /// protocol version, which changes after [CasperVm::enable_addressable_entity].
    fn advance_era(&mut self) {
        let step_request = self
            .context
            .step_request_builder()
            .with_protocol_version(self.protocol_version)
            .with_run_auction(true)
            .with_next_era_id(self.context.get_era().successor())
            .build();
        match self.context.step(step_request) {
            StepResult::Success { .. } => {}
            result => panic!("Failed to advance era: {:?}", result)
        }
    }

    /// Gets the time between auctions.
    pub fn auction_delay(&mut self) -> u64 {
        self.context
            .get_auction_delay()
            .saturating_mul(self.context.chainspec().core_config.era_duration.millis())
    }

    /// Returns the unbonding delay.
    pub fn unbonding_delay(&mut self) -> u64 {
        self.context
            .get_unbonding_delay()
            .saturating_mul(self.context.chainspec().core_config.era_duration.millis())
    }

    /// Returns the delegated amount.
    pub fn delegated_amount(&mut self, delegator: Address, validator: PublicKey) -> U512 {
        let purse_uref = self.get_main_purse(delegator);
        let account_hash = validator.to_account_hash();

        let bid = self
            .context
            .get_bids()
            .into_iter()
            .find(|bid| bid.validator_public_key() == validator && bid.is_delegator());

        match bid {
            None => U512::zero(),
            Some(bid_kind) => bid_kind.staked_amount().unwrap_or_default()
        }
    }

    fn validator_staked_amount(&mut self, validator: PublicKey) -> U512 {
        self.context
            .get_bids()
            .into_iter()
            .filter(|bid| bid.validator_public_key() == validator && !bid.is_delegator())
            .fold(U512::zero(), |acc, bid| {
                acc + bid.staked_amount().unwrap_or_default()
            })
    }

    /// Disables the validator.
    /// Undelegates the validator's stakes.
    pub fn remove_validator(&mut self, validator: PublicKey) {
        // Since casper 2.1 withdrawing more than the validator's own stake is an error
        // (`UnbondTooLarge`); zeroing the validator bid undelegates its delegators.
        let amount = self.validator_staked_amount(validator.clone());
        let withdraw_request = ExecuteRequestBuilder::contract_call_by_hash(
            validator.to_account_hash(),
            self.context.get_auction_contract_hash(),
            METHOD_WITHDRAW_BID,
            runtime_args! {
                ARG_PUBLIC_KEY => validator.clone(),
                ARG_AMOUNT => amount,
            }
        )
        .with_protocol_version(self.protocol_version)
        .build();

        self.context
            .exec(withdraw_request)
            .commit()
            .expect_success();

        self.removed_validators.push(validator.clone());
    }

    /// Switches the VM from legacy mode to addressable-entity mode, replaying the
    /// production migration path: the global state is reopened with an
    /// entity-enabled chainspec and a protocol upgrade migrates the system
    /// contracts. User accounts and user contracts stay in their legacy form and
    /// are migrated lazily on first use, just like on a real network.
    ///
    /// Returns `false` (and changes nothing) if the VM already runs in
    /// addressable-entity mode. Boot the VM with `ODRA_CASPER_LEGACY_GENESIS=1`
    /// to start in legacy mode.
    pub fn enable_addressable_entity(&mut self) -> bool {
        if self.chainspec.core_config.enable_addressable_entity {
            return false;
        }

        let post_state_hash = self.context.get_post_state_hash();
        self.context.flush_environment();
        // The addressable-entity flag is baked into the builder's data access
        // layer at construction time, so the only way to change it is to reopen
        // the global state with a new configuration. LMDB must not be opened
        // twice within one process - drop the old builder first.
        let old_builder = std::mem::replace(&mut self.context, LmdbWasmTestBuilder::default());
        drop(old_builder);

        let chainspec = self.chainspec.clone().with_enable_addressable_entity(true);
        let mut builder = LmdbWasmTestBuilder::open(
            self.data_dir.path(),
            chainspec.clone(),
            self.protocol_version,
            post_state_hash
        );

        let next_protocol_version =
            ProtocolVersion::from_parts(self.protocol_version.value().major + 1, 0, 0);
        let mut upgrade_request = UpgradeRequestBuilder::new()
            .with_current_protocol_version(self.protocol_version)
            .with_new_protocol_version(next_protocol_version)
            .with_activation_point(builder.get_era().successor())
            .with_new_gas_hold_handling(HoldBalanceHandling::Accrued)
            .with_new_gas_hold_interval(24 * 60 * 60 * 60)
            .with_enable_addressable_entity(true)
            .build();
        builder
            .with_block_time(BlockTime::new(self.block_time))
            .upgrade_using_scratch(&mut upgrade_request)
            .expect_upgrade_success();

        self.context = builder;
        self.chainspec = chainspec;
        self.protocol_version = next_protocol_version;
        true
    }

    fn get_main_purse(&self, address: Address) -> URef {
        match address {
            Address::Account(account) => {
                // Works in both legacy and addressable-entity mode.
                let entity = self.context.get_entity_by_account_hash(account).unwrap_or_else(|| {
                    panic!("Account not found while getting entity addr: {:?}", account)
                });
                entity.main_purse()
            }
            Address::Contract(contract) => self
                .get_contract_main_purse(PackageHash::new(contract.value()))
                .unwrap_or_else(|| {
                    panic!(
                        "Contract purse not found while getting entity addr: {:?}",
                        contract
                    )
                })
        }
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
        let package_hash = contract_address.as_package_hash().unwrap();

        let dictionary_seed_uref = self.package_named_key(package_hash, EVENTS);

        match dictionary_seed_uref {
            None => Err(EventError::CouldntExtractEventData),
            Some(uref) => Ok(self.get_dict_value(*uref.as_uref().unwrap(), &index.to_string()))
        }
    }

    /// Gets the native event at the specified index for the given contract address.
    ///
    /// TODO: Support negative index
    /// The index may be negative, in which case it is interpreted as an offset from the end of the event list.
    ///
    /// Returns [EventError::IndexOutOfBounds] if the index is out of bounds.
    pub fn get_native_event(
        &self,
        contract_address: &Address,
        index: u32
    ) -> Result<Bytes, EventError> {
        let messages = self
            .messages
            .get(&self.get_contract_entity_addr(contract_address))
            .ok_or(EventError::IndexOutOfBounds)?;
        let message = messages
            .get(index as usize)
            .ok_or(EventError::IndexOutOfBounds)?;
        match message {
            MessagePayload::String(_) => Err(EventError::CouldntExtractEventData),
            MessagePayload::Bytes(b) => Ok(b.clone())
        }
    }

    /// Gets the count of events for the given contract address.
    pub fn get_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        let package_hash = contract_address.as_package_hash();
        if package_hash.is_none() {
            return Err(EventError::TriedToQueryEventForNonContract);
        }
        Ok(self.events_length(package_hash.unwrap()))
    }

    /// Gets the count of native events for the given contract address.
    pub fn get_native_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        let messages = self
            .messages
            .get(&self.get_contract_entity_addr(contract_address))
            .ok_or(EventError::IndexOutOfBounds)?;
        Ok(messages.len() as u32)
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
        let hash = address
            .as_contract_package_hash()
            .expect("Contract hash expected");

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
            .with_protocol_version(self.protocol_version)
            .build();
        self.context.exec(execute_request).commit();
        self.collect_gas();
        self.gas_report.push(DeployReport::ContractCall {
            gas: self.last_call_contract_gas_cost(),
            contract_address: *address,
            call_def: call_def.clone()
        });

        self.collect_messages();

        self.attached_value = U512::zero();
        if let Some(error) = self.context.get_error() {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            self.panic_with_error(odra_error, call_def.entry_point(), hash);
        } else {
            self.get_active_account_result()
        }
    }

    fn collect_messages(&mut self) {
        let messages = self.context.get_last_exec_result().unwrap();
        let messages = messages.messages();
        messages.iter().for_each(|message| {
            let payload = message.payload().clone();
            self.messages
                .entry(*message.entity_addr())
                .or_default()
                .push(payload);
        });
    }

    fn get_contract_entity_addr(&self, address: &Address) -> EntityAddr {
        match address {
            Address::Account(_) => {
                panic!(
                    "Account address passed instead of contract address: {:?}",
                    address
                )
            }
            Address::Contract(contract) => {
                let package = self
                    .query_package(PackageHash::new(contract.value()))
                    .unwrap_or_else(|| {
                        panic!(
                            "Contract package not found while getting entity addr: {:?}",
                            contract
                        )
                    });

                package.current_entity_hash().unwrap_or_else(|| {
                    panic!(
                        "Current entity hash not found while getting entity addr: {:?}",
                        contract
                    )
                })
            }
        }
    }

    fn get_addressable_entity_from_entity_addr(
        &self,
        entity_addr: &EntityAddr
    ) -> AddressableEntity {
        let query_result = self
            .context
            .query(None, Key::AddressableEntity(*entity_addr), &[])
            .unwrap();
        if let StoredValue::AddressableEntity(entity) = query_result {
            entity
        } else {
            panic!(
                "Stored value is not an addressable entity: {:?}",
                query_result
            );
        }
    }

    /// Creates a new contract with the specified name, initialisation arguments, and entry points caller.
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

        let result = self.deploy_wasm(&wasm_path, &init_args);
        if let Some(error) = result {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            panic!("Revert: Contract deploy failed {:?}", odra_error);
        } else {
            let package_hash = self.package_hash_from_name(&package_hash_key_name);
            self.collect_messages();
            package_hash.into()
        }
    }

    /// Upgrades an existing contract with the specified name, initialisation arguments, and entry points caller.
    pub fn upgrade_contract(
        &mut self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let wasm_path = format!("{}.wasm", name);
        let result = self.deploy_wasm(&wasm_path, &upgrade_args);
        if let Some(error) = result {
            let odra_error = parse_error(error);
            self.error = Some(odra_error.clone());
            panic!("Revert: Contract deploy failed {:?}", odra_error);
        } else {
            self.collect_messages();
            contract_to_upgrade
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
            Address::Contract(package_hash) => {
                self.get_contract_cspr_balance(&address.as_package_hash().unwrap())
            }
        }
    }

    /// Transfers the specified number of tokens to the given address.
    ///
    /// Results an OdraError if the transfer fails.
    pub fn transfer(&mut self, to: Address, amount: U512) -> OdraResult<()> {
        let transfer_request = match to {
            Address::Account(account_hash) => TransferRequestBuilder::new(amount, account_hash),
            Address::Contract(_) => TransferRequestBuilder::new(amount, self.get_main_purse(to))
        }
        .with_initiator(self.active_account_hash())
        .with_transfer_id(0u64)
        .build();
        self.context.transfer_and_commit(transfer_request);

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
        self.context.last_exec_gas_consumed().value()
    }

    /// Returns the amount of gas used for the last call.
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

    fn get_contract_cspr_balance(&self, package_hash: &PackageHash) -> U512 {
        // TODO: Addressable entity has main purse inside it, is it the same as ours for contracts?
        let purse_uref = self.get_contract_main_purse(*package_hash);
        match purse_uref {
            None => U512::zero(),
            Some(uref) => self.context.get_purse_balance(uref)
        }
    }

    fn get_contract_main_purse(&self, package_hash: PackageHash) -> Option<URef> {
        let purse_key = self.package_named_key(package_hash, CONTRACT_MAIN_PURSE);
        match purse_key {
            None => None,
            Some(purse_key) => {
                let purse_uref = purse_key.as_uref().unwrap_or_else(|| {
                    panic!(
                        "Contract doesn't have main purse uref under {} named key",
                        CONTRACT_MAIN_PURSE
                    )
                });
                Some(*purse_uref)
            }
        }
    }

    fn genesis_accounts(
        key_pairs: &BTreeMap<Address, (SecretKey, PublicKey)>
    ) -> (Vec<GenesisAccount>, Vec<GenesisAccount>) {
        let mut accounts = Vec::new();
        let mut validators = Vec::new();
        let total_accounts = key_pairs.len();
        let validators_count = 5; // Fixed the number of validators
        let regular_accounts_count = total_accounts - validators_count;

        // Create regular accounts
        let iter = key_pairs.iter();
        for (_, (_, public_key)) in iter.take(regular_accounts_count) {
            accounts.push(GenesisAccount::account(
                public_key.clone(),
                Motes::new(DEFAULT_BALANCE),
                None
            ));
        }

        // Create validator accounts (last 5 accounts)
        for (_, (_, public_key)) in key_pairs.iter().skip(regular_accounts_count) {
            let validator_account = GenesisAccount::account(
                public_key.clone(),
                Motes::new(DEFAULT_BALANCE),
                Some(GenesisValidator::new(Motes::new(DEFAULT_BID_AMOUNT), 0))
            );
            accounts.push(validator_account.clone());
            validators.push(validator_account);
        }

        (accounts, validators)
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
        // Addressable entity is on by default; set ODRA_CASPER_LEGACY_GENESIS=1
        // to boot in legacy mode, e.g. to test contracts across the
        // `enable_addressable_entity` switch (see [CasperVm::enable_addressable_entity]).
        let legacy_genesis = env::var("ODRA_CASPER_LEGACY_GENESIS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self::new_instance_with_mode(legacy_genesis)
    }

    fn new_instance_with_mode(legacy_genesis: bool) -> Self {
        let key_pairs = generate_key_pairs(ACCOUNTS_NUMBER);
        let (genesis_accounts, validators) = Self::genesis_accounts(&key_pairs);
        let accounts: Vec<Address> = key_pairs.keys().copied().collect();

        let chainspec = ChainspecConfig::from_chainspec_path(&*CHAINSPEC_SYMLINK)
            .unwrap()
            .with_enable_addressable_entity(!legacy_genesis);
        let genesis_request = chainspec
            .create_genesis_request(genesis_accounts.clone(), ProtocolVersion::V2_0_0)
            .unwrap();

        let data_dir = Rc::new(tempfile::tempdir().expect("should create data dir"));
        let mut builder = LmdbWasmTestBuilder::new_with_config(data_dir.path(), chainspec.clone());
        builder.run_genesis(genesis_request).commit();
        let unbonding_delay = builder.get_unbonding_delay();
        let auction_delay = builder.get_auction_delay();
        builder.advance_eras_by(unbonding_delay + auction_delay);

        for account in validators.iter() {
            let bid_request = ExecuteRequestBuilder::contract_call_by_hash(
                account.account_hash(),
                builder.get_auction_contract_hash(),
                METHOD_ADD_BID,
                runtime_args! {
                    ARG_PUBLIC_KEY => account.public_key(),
                    ARG_AMOUNT => U512::from(DEFAULT_BID_AMOUNT),
                    ARG_DELEGATION_RATE=> 0u8,
                    ARG_MINIMUM_DELEGATION_AMOUNT => DEFAULT_MINIMUM_DELEGATION_AMOUNT,
                }
            )
            .build();

            builder.exec(bid_request).commit().expect_success();
        }

        Self {
            active_account: accounts[0],
            context: builder,
            data_dir,
            chainspec,
            protocol_version: ProtocolVersion::V2_0_0,
            accounts,
            block_time: 0u64,
            calls_counter: 0,
            error: None,
            attached_value: U512::zero(),
            gas_used: BTreeMap::new(),
            gas_report: GasReport::default(),
            key_pairs,
            messages: Default::default(),
            validators,
            removed_validators: Default::default()
        }
    }

    /// Deploys a session wasm from the given path with the given args,
    /// signed by the active account. Returns the execution error, if any.
    pub fn deploy_wasm(&mut self, wasm_path: &str, args: &RuntimeArgs) -> Option<engine_state::Error> {
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
            .with_protocol_version(self.protocol_version)
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
    fn get_package(&self, package_hash: PackageHash) -> Package {
        self.query_package(package_hash)
            .unwrap_or_else(|| panic!("Package not found: {:?}", package_hash))
    }

    /// Reads a package from the global state, regardless of its storage form.
    /// After [CasperVm::enable_addressable_entity] a package installed in legacy
    /// mode remains under [Key::Hash] until its first call migrates it, so both
    /// locations must be checked.
    fn query_package(&self, package_hash: PackageHash) -> Option<Package> {
        let keys = [
            Key::SmartContract(package_hash.value()),
            Key::Hash(package_hash.value())
        ];
        for key in keys {
            match self.context.query(None, key, &[]) {
                Ok(StoredValue::SmartContract(package)) => return Some(package),
                Ok(StoredValue::ContractPackage(contract_package)) => {
                    return Some(contract_package.into())
                }
                _ => {}
            }
        }
        None
    }

    /// Gets current contract from contract package and
    /// returns it's named keys.
    fn package_named_keys(&self, package_hash: PackageHash) -> NamedKeys {
        let current_entity_hash = self
            .get_package(package_hash)
            .current_entity_hash()
            .unwrap_or_else(|| panic!("Package has no current version: {:?}", package_hash));
        let addressable_entity_hash = AddressableEntityHash::new(current_entity_hash.value());
        if let Some(entity) = self
            .context
            .get_entity_with_named_keys_by_entity_hash(addressable_entity_hash)
        {
            return entity.named_keys().clone();
        }
        // A legacy contract not yet migrated to an entity.
        match self.context.query(None, Key::Hash(addressable_entity_hash.value()), &[]) {
            Ok(StoredValue::Contract(contract)) => contract.take_named_keys(),
            other => panic!("Contract not found: {:?}", other)
        }
    }

    fn package_named_key(&self, package_hash: PackageHash, name: &str) -> Option<Key> {
        self.package_named_keys(package_hash).get(name).cloned()
    }

    fn events_length(&self, package_hash: PackageHash) -> u32 {
        let key = self.package_named_key(package_hash, EVENTS_LENGTH);
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

    fn panic_with_error(
        &self,
        error: OdraError,
        entrypoint: &str,
        package_hash: &ContractPackageHash
    ) -> ! {
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
                x if x == ExecutionError::CannotOverrideKeys.code() => {
                    OdraError::ExecutionError(ExecutionError::CannotOverrideKeys)
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
                // The exact error name is not known, so we return a generic error.
                _ => OdraError::user(code, CASPER_ERROR_GENERIC_NAME)
            },
            execution::ExecError::InvalidContext => OdraError::VmError(VmError::InvalidContext),
            execution::ExecError::NoSuchMethod(name) => {
                OdraError::VmError(VmError::NoSuchMethod(name))
            }
            execution::ExecError::MissingArgument { name } => {
                OdraError::ExecutionError(ExecutionError::MissingArg)
            }
            _ => OdraError::VmError(VmError::Other(format!("Casper ExecError: {exec_err}")))
        }
    } else {
        OdraError::VmError(VmError::Other(format!("Casper EngineStateError: {}", err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// After the addressable-entity switch, a lazily migrated account must keep
    /// its identity: the entity address carries the original account hash bytes
    /// and the main purse (thus the CSPR balance) is preserved.
    #[test]
    fn account_entity_keeps_account_hash_across_ae_switch() {
        let mut vm = CasperVm::new_instance_with_mode(true);

        let sender = vm.accounts[0];
        let recipient = vm.accounts[1];
        let sender_hash = vm.active_account_hash();
        let pre_purse = vm.get_main_purse(sender);
        let pre_balance = vm.balance_of(&recipient);

        // Pre-flip the account is a legacy record.
        let stored = vm.context.query(None, Key::Account(sender_hash), &[]).unwrap();
        assert!(matches!(stored, StoredValue::Account(_)));

        assert!(vm.enable_addressable_entity());
        assert!(!vm.enable_addressable_entity(), "second switch must be a no-op");

        // First use after the switch migrates the account lazily.
        let amount = U512::from(1_000_000_000_000u64);
        vm.transfer(recipient, amount).unwrap();
        assert_eq!(vm.balance_of(&recipient), pre_balance + amount);

        // Key::Account now points at an entity that carries the account hash bytes.
        let stored = vm.context.query(None, Key::Account(sender_hash), &[]).unwrap();
        let entity_key = match stored {
            StoredValue::CLValue(cl_value) => cl_value.into_t::<Key>().unwrap(),
            other => panic!("expected CLValue indirection, got {:?}", other)
        };
        let entity_addr = match entity_key {
            Key::AddressableEntity(entity_addr) => entity_addr,
            other => panic!("expected Key::AddressableEntity, got {:?}", other)
        };
        assert_eq!(entity_addr, EntityAddr::Account(sender_hash.value()));

        // The main purse is carried over.
        assert_eq!(vm.get_main_purse(sender), pre_purse);
    }

    /// Replays the compatibility concern raised for the upstream
    /// casper-ecosystem/cep18 reference implementation (v2.0.0): tokens held by
    /// a *contract* are keyed by its `get_immediate_caller()` result, which
    /// under legacy rules resolves to `Key::Hash(<contract package>)` but under
    /// addressable-entity mode resolves to
    /// `Key::AddressableEntity(<contract entity>)` - a different key. Funds a
    /// contract deposited before the switch therefore become inaccessible to it
    /// after the switch, while account-held balances keep working.
    ///
    /// If a fixed cep18 release changes this behavior, update the vendored
    /// wasm files (see resources/README.md) and the final assertions.
    #[test]
    fn upstream_cep18_across_ae_switch() {
        use odra_core::casper_types::U256;

        const TOKEN_NAME: &str = "TestCoin";
        let resources = concat!(env!("CARGO_MANIFEST_DIR"), "/resources");

        let mut vm = CasperVm::new_instance_with_mode(true);
        let deployer = vm.active_account_hash();
        let alice = vm.accounts[1];
        let alice_key = Key::Account(*alice.as_account_hash().unwrap());

        // Install the upstream token and its test (client) contract.
        let error = vm.deploy_wasm(
            &format!("{}/cep18.wasm", resources),
            &runtime_args! {
                "name" => TOKEN_NAME,
                "symbol" => "TC",
                "decimals" => 8u8,
                "total_supply" => U256::from(1_000_000u64),
            }
        );
        assert!(error.is_none(), "cep18 install failed: {:?}", error);
        let error = vm.deploy_wasm(
            &format!("{}/cep18_test_contract.wasm", resources),
            &RuntimeArgs::new()
        );
        assert!(error.is_none(), "test contract install failed: {:?}", error);

        let named_keys = vm.context.get_named_keys_by_account_hash(deployer);
        let get_hash = |name: &str| -> HashAddr {
            named_keys
                .get(name)
                .unwrap_or_else(|| panic!("missing named key {}", name))
                .into_hash_addr()
                .unwrap_or_else(|| panic!("named key {} is not a hash", name))
        };
        let token_contract = get_hash(&format!("cep18_contract_hash_{}", TOKEN_NAME));
        let token_contract_key = Key::Hash(token_contract);
        let client_package = get_hash("cep18_test_contract_package_hash");
        let client_package_key = Key::Hash(client_package);

        let mut call = |vm: &mut CasperVm, contract: HashAddr, entry_point: &str, args: RuntimeArgs| {
            let request = ExecuteRequestBuilder::contract_call_by_hash(
                deployer,
                AddressableEntityHash::new(contract),
                entry_point,
                args
            )
            .with_protocol_version(vm.protocol_version)
            .build();
            vm.context.exec(request).commit();
            vm.context.get_error()
        };
        let client_contract = get_hash("cep18_test_contract_hash");
        let read_result = |vm: &CasperVm| -> U256 {
            let key = vm
                .package_named_key(PackageHash::new(client_package), "result")
                .expect("result key");
            vm.get_value(key)
        };
        let check_balance = |vm: &mut CasperVm,
                             call: &mut dyn FnMut(
            &mut CasperVm,
            HashAddr,
            &str,
            RuntimeArgs
        ) -> Option<engine_state::Error>,
                             address: Key|
         -> U256 {
            let error = call(
                vm,
                client_contract,
                "check_balance_of",
                runtime_args! { "token_contract" => token_contract_key, "address" => address }
            );
            assert!(error.is_none(), "check_balance_of failed: {:?}", error);
            read_result(vm)
        };

        // Pre-switch: fund the client contract and let it spend as a contract.
        let error = call(
            &mut vm,
            token_contract,
            "transfer",
            runtime_args! { "recipient" => client_package_key, "amount" => U256::from(1_000u64) }
        );
        assert!(error.is_none(), "account transfer failed: {:?}", error);
        let error = call(
            &mut vm,
            client_contract,
            "transfer_as_stored_contract",
            runtime_args! {
                "token_contract" => token_contract_key,
                "recipient" => alice_key,
                "amount" => U256::from(400u64)
            }
        );
        assert!(error.is_none(), "contract transfer failed: {:?}", error);
        assert_eq!(
            check_balance(&mut vm, &mut call, client_package_key),
            U256::from(600u64)
        );
        assert_eq!(check_balance(&mut vm, &mut call, alice_key), U256::from(400u64));

        assert!(vm.enable_addressable_entity());

        // Account-held balances keep working across the switch.
        assert_eq!(check_balance(&mut vm, &mut call, alice_key), U256::from(400u64));
        let error = call(
            &mut vm,
            token_contract,
            "transfer",
            runtime_args! { "recipient" => alice_key, "amount" => U256::from(100u64) }
        );
        assert!(error.is_none(), "post-switch account transfer failed: {:?}", error);
        assert_eq!(check_balance(&mut vm, &mut call, alice_key), U256::from(500u64));

        // The contract's pre-switch balance is still recorded under its legacy key...
        assert_eq!(
            check_balance(&mut vm, &mut call, client_package_key),
            U256::from(600u64)
        );

        // ...but the contract can no longer spend it: its caller identity now
        // resolves to `Key::AddressableEntity(<entity>)`, whose balance is zero,
        // so the transfer fails with Cep18Error::InsufficientBalance (60001).
        let error = call(
            &mut vm,
            client_contract,
            "transfer_as_stored_contract",
            runtime_args! {
                "token_contract" => token_contract_key,
                "recipient" => alice_key,
                "amount" => U256::from(100u64)
            }
        );
        assert!(
            error.is_some(),
            "expected the upstream cep18 contract-caller transfer to fail after \
             the addressable-entity switch; it succeeded - the upstream bug may \
             have been fixed, update the vendored wasm and these assertions"
        );
        // The contract's funds remain locked under the legacy key.
        assert_eq!(
            check_balance(&mut vm, &mut call, client_package_key),
            U256::from(600u64)
        );
    }
}
