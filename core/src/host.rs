//! A module that provides the interface for interacting with the host environment.

#[cfg(not(target_arch = "wasm32"))]
mod concurrently;
mod context;
mod deployed_contracts;
mod deployer;
mod events;
pub mod factory;
#[cfg(not(target_arch = "wasm32"))]
pub mod revert_hook;

#[cfg(not(target_arch = "wasm32"))]
pub use concurrently::ThreadEnvFactory;
pub use context::HostContext;
#[cfg(test)]
pub(crate) use context::MockHostContext;
#[cfg(not(target_arch = "wasm32"))]
pub use deployer::{Deployer, InstallConfig, UpgradeConfig};
pub use deployer::{InitArgs, NoArgs, UpgradeArgs};

use crate::address::Addressable;
use crate::gas_report::GasReport;
use crate::host::deployed_contracts::DeployedContract;
use crate::prelude::*;
use crate::{
    call_result::CallResult, entry_point_callback::EntryPointsCaller, CallDef, ContractCallResult,
    ContractEnv, EventError
};
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{Bytes, FromBytes},
    CLTyped, PublicKey, RuntimeArgs, U512
};
use core::time::Duration;

/// A host side reference to a contract.
pub trait HostRef {
    /// Creates a new host side reference to a contract.
    ///
    /// A generated host ref registers the contract at `address` in `env`, like
    /// [HostRefLoader::load] does, see [HostEnv::register_contract_ref].
    fn new(address: Address, env: HostEnv) -> Self;
    /// Creates a new host reference with attached tokens, based on the current instance.
    ///
    /// If there are tokens attached to the current instance, the tokens will be attached
    /// to the next contract call.
    fn with_tokens(&self, tokens: U512) -> Self;
    /// Returns the address of the contract.
    fn contract_address(&self) -> Address;
    /// Returns the host environment.
    fn env(&self) -> &HostEnv;
    /// Returns the n-th event emitted by the contract.
    ///
    /// If the event is not found or the type does not match, returns `EventError::EventNotFound`.
    fn get_event<T>(&self, index: i32) -> Result<T, EventError>
    where
        T: FromBytes + EventInstance + 'static;
    /// Returns a detailed information about the last call of the contract.
    fn last_call(&self) -> ContractCallResult;
}

impl<T: HostRef> Addressable for T {
    fn address(&self) -> Address {
        HostRef::contract_address(self)
    }
}

/// Trait for loading a contract from the host environment.
///
/// Similar to [Deployer], but does not deploy a new contract, but loads an existing one.
pub trait HostRefLoader<T: HostRef> {
    /// Loads an existing contract from the host environment.
    fn load(env: &HostEnv, address: Address) -> T;
}

/// A type which can provide an [EntryPointsCaller].
pub trait EntryPointsCallerProvider {
    /// Returns the [EntryPointsCaller] of the contract.
    fn entry_points_caller() -> EntryPointsCaller;
}

/// Represents the host environment for executing smart contracts.
///
/// It provides methods for interacting with the underlying host context and managing
/// the execution of contracts.
#[derive(Clone)]
pub struct HostEnv {
    backend: Rc<dyn HostContext>,
    last_call_result: Rc<RefCell<Option<CallResult>>>,
    deployed_contracts: Rc<RefCell<BTreeMap<Address, DeployedContract>>>,
    /// The deployed contracts remembered by [take_snapshot](Self::take_snapshot).
    deployed_contracts_snapshot: Rc<RefCell<Option<BTreeMap<Address, DeployedContract>>>>,
    captures_events: Rc<RefCell<bool>>
}

impl HostEnv {
    /// Creates a new `HostEnv` instance with the specified backend.
    pub fn new(backend: Rc<dyn HostContext>) -> HostEnv {
        HostEnv {
            backend,
            last_call_result: RefCell::new(None).into(),
            deployed_contracts: RefCell::new(Default::default()).into(),
            deployed_contracts_snapshot: RefCell::new(None).into(),
            captures_events: Rc::new(RefCell::new(true))
        }
    }

    /// Returns the account address at the specified index.
    pub fn get_account(&self, index: usize) -> Address {
        let backend = self.backend.as_ref();
        backend.get_account(index)
    }

    /// Returns the validator public key.
    pub fn get_validator(&self, index: usize) -> PublicKey {
        let backend = self.backend.as_ref();
        backend.get_validator(index)
    }

    /// Sets the caller address for the current contract execution.
    pub fn set_caller(&self, address: Address) {
        if address.is_contract() {
            panic!("Caller cannot be a contract: {:?}", address)
        }
        let backend = self.backend.as_ref();
        backend.set_caller(address)
    }

    /// Advances the block time by `time_diff`: a [Duration] or a number of milliseconds.
    ///
    /// Block time has millisecond resolution; anything finer is truncated.
    ///
    /// ```
    /// # use core::time::Duration;
    /// # fn shift(env: &odra_core::host::HostEnv) {
    /// env.advance_block_time(Duration::from_secs(60 * 60 * 24));
    /// env.advance_block_time(1_000); // milliseconds
    /// # }
    /// ```
    pub fn advance_block_time(&self, time_diff: impl BlockTimeDiff) {
        let backend = self.backend.as_ref();
        backend.advance_block_time(time_diff.into_millis())
    }

    /// Advances the block time by `time_diff` (a [Duration] or a number of milliseconds) and
    /// processes auctions.
    ///
    /// Block time has millisecond resolution; anything finer is truncated.
    pub fn advance_with_auctions(&self, time_diff: impl BlockTimeDiff) {
        let backend = self.backend.as_ref();
        backend.advance_with_auctions(time_diff.into_millis());
    }

    /// Returns the era length in milliseconds.
    pub fn auction_delay(&self) -> u64 {
        let backend = self.backend.as_ref();
        backend.auction_delay()
    }

    /// Returns the delay between unstaking and the transfer of funds back to the delegator in milliseconds.
    pub fn unbonding_delay(&self) -> u64 {
        let backend = self.backend.as_ref();
        backend.unbonding_delay()
    }

    /// Returns the amount of CSPR delegated to the specified validator by the specified delegator.
    pub fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        let backend = self.backend.as_ref();
        backend.delegated_amount(delegator, validator)
    }

    /// Evicts the validator at the specified index from the validator set.
    pub fn remove_validator(&self, index: usize) {
        let backend = self.backend.as_ref();
        backend.remove_validator(index);
    }

    /// Returns the current block time in milliseconds.
    pub fn block_time(&self) -> u64 {
        let backend = self.backend.as_ref();
        backend.block_time()
    }

    /// Returns the current block time in milliseconds.
    pub fn block_time_millis(&self) -> u64 {
        let backend = self.backend.as_ref();
        backend.block_time()
    }

    /// Returns the current block time in seconds.
    pub fn block_time_secs(&self) -> u64 {
        let backend = self.backend.as_ref();
        backend.block_time() / 1000
    }

    /// Remembers the current state of the test VM: deployed contracts and their code, contract
    /// storage, CSPR balances, events and the block time.
    ///
    /// [`restore_snapshot`](Self::restore_snapshot) brings that state back, as many times as
    /// needed, so several scenarios can branch off one expensive setup. Only the last snapshot
    /// is kept: taking a new one replaces it. The caller chosen with
    /// [`set_caller`](Self::set_caller) and the gas report are not part of a snapshot.
    ///
    /// Available on OdraVM and CasperVM; livenet has no snapshots.
    ///
    /// ```
    /// # use odra_core::host::HostEnv;
    /// # fn scenarios(env: &HostEnv, run_scenario_a: impl Fn(), run_scenario_b: impl Fn()) {
    /// // ... deploy and configure the contracts ...
    /// env.take_snapshot();
    /// run_scenario_a();
    /// env.restore_snapshot();
    /// run_scenario_b();
    /// # }
    /// ```
    pub fn take_snapshot(&self) {
        let backend = self.backend.as_ref();
        backend.take_snapshot();
        let deployed_contracts = self.deployed_contracts.borrow().clone();
        *self.deployed_contracts_snapshot.borrow_mut() = Some(deployed_contracts);
    }

    /// Brings the test VM back to the state remembered by the last
    /// [`take_snapshot`](Self::take_snapshot).
    ///
    /// # Panics
    ///
    /// Panics if no snapshot has been taken.
    pub fn restore_snapshot(&self) {
        let backend = self.backend.as_ref();
        backend.restore_snapshot();
        // The contracts deployed since are gone, the versions and event counts are back.
        if let Some(deployed_contracts) = self.deployed_contracts_snapshot.borrow().as_ref() {
            *self.deployed_contracts.borrow_mut() = deployed_contracts.clone();
        }
    }

    /// Registers a new contract with the specified name, initialization arguments, and entry points caller.
    pub fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        // Filter "upgrade" from EntryPointsCaller
        let mut entry_points_caller = entry_points_caller.clone();
        entry_points_caller.remove_entry_point("upgrade");

        let backend = self.backend.as_ref();
        let result = backend.new_contract(name, init_args, entry_points_caller);
        self.track_child_contracts(result.is_ok());
        let contract_address = result?;

        self.deployed_contracts.borrow_mut().insert(
            contract_address,
            DeployedContract::with_name(contract_address, name)
        );
        // The events of `init` belong to the deploy, not to the first call after it.
        if *self.captures_events.borrow() {
            self.init_events(&contract_address);
        }
        Ok(contract_address)
    }

    /// Upgrades an existing contract with a new one with given upgrade arguments and new entry
    /// points caller.
    ///
    /// The contract need not be known to this environment (deployed, loaded or wrapped in a host
    /// ref here): the upgrade registers it. The events of a known contract, native ones included,
    /// are counted over all its versions, so its counts carry over the upgrade.
    pub fn upgrade_contract(
        &self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        // Filter "init" from EntryPointsCaller
        let mut entry_points_caller = entry_points_caller.clone();
        entry_points_caller.remove_entry_point("init");

        let backend = self.backend.as_ref();
        let result =
            backend.upgrade_contract(name, contract_to_upgrade, upgrade_args, entry_points_caller);
        self.track_child_contracts(result.is_ok());
        let upgraded_contract = result?;
        let newly_known = {
            let mut contracts = self.deployed_contracts.borrow_mut();
            let newly_known = !contracts.contains_key(&upgraded_contract);
            let contract = contracts
                .entry(upgraded_contract)
                .or_insert_with(|| DeployedContract::new(upgraded_contract));
            contract.current_version += 1;
            contract.name = Some(String::from(name));
            newly_known
        };
        // The events emitted before a contract was known here belong to nobody's `last_call`.
        if newly_known && *self.captures_events.borrow() {
            self.init_events(&upgraded_contract);
        }
        Ok(upgraded_contract)
    }

    /// Registers an existing contract with the specified address, name and entry points caller.
    /// Similar to `new_contract`, but skips the deployment phase.
    ///
    /// Registering a contract this environment already knows keeps its events: the events
    /// emitted so far still belong to the calls that emitted them.
    pub fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        let backend = self.backend.as_ref();
        backend.register_contract(address, contract_name.clone(), entry_points_caller);
        self.deployed_contracts
            .borrow_mut()
            .entry(address)
            .or_insert_with(|| DeployedContract::new(address))
            .name = Some(contract_name);
        // Events emitted before the contract was loaded belong to nobody's `last_call`.
        if *self.captures_events.borrow() {
            self.init_events(&address);
        }
    }

    /// Registers the contract a host ref is made for with [HostRef::new], so that a host ref
    /// made from a bare address (e.g. of a contract deployed by a factory) works like one made
    /// by [HostRefLoader::load]. On livenet, the non-mutable calls run on the host and need the
    /// contract code.
    ///
    /// Does nothing if the contract is already registered under `contract_name`, and nothing
    /// for an address that holds no contract on the backend (see
    /// [HostContext::has_contract]): a host ref to a made-up address does not make up a
    /// contract, its calls fail.
    pub fn register_contract_ref(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: impl FnOnce() -> EntryPointsCaller
    ) {
        let known = match self.deployed_contracts.borrow().get(&address) {
            Some(contract) if contract.name.as_ref() == Some(&contract_name) => return,
            Some(_) => true,
            None => false
        };
        if known || self.backend.has_contract(&address) {
            self.register_contract(address, contract_name, entry_points_caller());
        }
    }

    /// Calls a contract at the specified address with the given call definition.
    pub fn call_contract<T: FromBytes + CLTyped>(
        &self,
        address: Address,
        call_def: CallDef
    ) -> OdraResult<T> {
        let use_proxy = T::cl_type() != <()>::cl_type() || !call_def.amount().is_zero();
        let call_result = self.raw_call_contract(address, call_def, use_proxy);
        call_result.map(|bytes| {
            T::from_bytes(&bytes)
                .map(|(obj, _)| obj)
                .map_err(OdraError::from)
        })?
    }

    /// Calls a contract at the specified address with the given call definition. Returns raw,
    /// not serialized bytes.
    pub fn raw_call_contract(
        &self,
        address: Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        let call_result = {
            let backend = self.backend.as_ref();
            backend.call_contract(&address, call_def, use_proxy)
        };
        self.track_child_contracts(call_result.is_ok());

        let mut events_map: BTreeMap<Address, Vec<Bytes>> = BTreeMap::new();
        let mut native_events_map: BTreeMap<Address, Vec<Bytes>> = BTreeMap::new();

        let captures_events = *self.captures_events.borrow();
        if captures_events {
            // Go through all contracts and collect their events
            self.deployed_contracts.borrow_mut().iter_mut().for_each(
                |(contract_address, contract)| {
                    let events = self.last_events(contract);
                    let native_events = self.last_native_events(contract);
                    events_map.insert(*contract_address, events);
                    native_events_map.insert(*contract_address, native_events);
                }
            );
        }

        let backend = self.backend.as_ref();
        let last_call_gas_cost = backend.last_call_gas_cost();

        self.last_call_result.replace(Some(CallResult::new(
            address,
            backend.caller(),
            last_call_gas_cost,
            call_result.clone(),
            events_map,
            native_events_map
        )));

        call_result
    }

    /// Returns the gas cost of the last contract call.
    pub fn contract_env(&self) -> ContractEnv {
        self.backend.contract_env()
    }

    /// Prints the gas report for the current contract execution.
    pub fn gas_report(&self) -> GasReport {
        self.backend.gas_report().clone()
    }

    /// Returns the CSPR balance of the specified address.
    pub fn balance_of<T: Addressable>(&self, addr: &T) -> U512 {
        let backend = self.backend.as_ref();
        backend.balance_of(&addr.address())
    }

    /// Returns the last call result for the specified contract.
    pub fn last_call_result(&self, contract_address: Address) -> ContractCallResult {
        self.last_call_result
            .borrow()
            .clone()
            .unwrap()
            .contract_last_call(contract_address)
    }

    /// Signs the specified message with the private key of the specified address.
    pub fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        let backend = self.backend.as_ref();
        backend.sign_message(message, address)
    }

    /// Returns the public key associated with the specified address.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        let backend = self.backend.as_ref();
        backend.public_key(address)
    }

    /// Returns the caller address for the current contract execution.
    pub fn caller(&self) -> Address {
        let backend = self.backend.as_ref();
        backend.caller()
    }

    /// Sets the gas limit for the current contract execution.
    pub fn set_gas(&self, gas: u64) {
        let backend = self.backend.as_ref();
        backend.set_gas(gas)
    }

    /// Reads a raw value from the Odra storage (the `state` dictionary) of the contract
    /// at the given address, without calling the contract.
    ///
    /// `key` is the storage key as produced by the contract environment (a hex-encoded hash).
    /// Use it together with the contract's storage layout to read the state directly.
    pub fn get_storage_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.backend.get_storage_value(address, key)
    }

    /// Reads the raw value stored under a named key of the contract at the given address.
    pub fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        self.backend.get_named_value(address, name)
    }

    /// Reads the raw value stored in a named dictionary of the contract at the given address.
    pub fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        self.backend
            .get_dictionary_value(address, dictionary_name, key)
    }

    /// Transfers the specified amount of CSPR from the current caller to the specified address.
    pub fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        if to.is_contract() {
            return Err(OdraError::ExecutionError(
                ExecutionError::TransferToContract
            ));
        }
        let backend = self.backend.as_ref();
        backend.transfer(to, amount)
    }

    /// Starts tracking the contracts the backend deployed from other contracts (factory
    /// children), unless the operation that deployed them failed and reverted them.
    fn track_child_contracts(&self, succeeded: bool) {
        let children = self.backend.take_child_contracts();
        if !succeeded {
            return;
        }
        let captures_events = *self.captures_events.borrow();
        let mut contracts = self.deployed_contracts.borrow_mut();
        for address in children {
            // The contract is new, all its events belong to the operation that deployed it.
            let mut contract = DeployedContract::new(address);
            contract.events_initialized = captures_events;
            contracts.insert(address, contract);
        }
    }
}

/// A block time shift: a [Duration] or a number of milliseconds (`u64`).
///
/// Block time is kept in milliseconds by every backend; a [Duration] is truncated to whole
/// milliseconds.
pub trait BlockTimeDiff {
    /// The shift in milliseconds.
    fn into_millis(self) -> u64;
}

impl BlockTimeDiff for u64 {
    fn into_millis(self) -> u64 {
        self
    }
}

impl BlockTimeDiff for Duration {
    fn into_millis(self) -> u64 {
        u64::try_from(self.as_millis()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use casper_types::account::AccountHash;
    use casper_types::contracts::ContractPackageHash;
    use mockall::predicate;

    #[test]
    fn test_host_env() {
        let mut ctx = MockHostContext::new();
        ctx.expect_new_contract()
            .returning(|_, _, _| Ok(Address::Account(AccountHash::new([0; 32]))));
        ctx.expect_take_child_contracts().returning(Vec::new);
        ctx.expect_caller()
            .returning(|| Address::Account(AccountHash::new([2; 32])))
            .times(1);
        ctx.expect_gas_report().returning(GasReport::new).times(1);
        ctx.expect_set_gas().returning(|_| ()).times(1);

        let env = HostEnv::new(Rc::new(ctx));

        assert_eq!(env.caller(), Address::Account(AccountHash::new([2; 32])));
        // should call the `HostContext`
        env.gas_report();
        env.set_gas(1_000u64)
    }

    #[test]
    fn test_successful_transfer_to_account() {
        // Given a host context that successfully transfers tokens.
        let mut ctx = MockHostContext::new();
        ctx.expect_transfer().returning(|_, _| Ok(()));
        let env = HostEnv::new(Rc::new(ctx));

        let addr = Address::Account(AccountHash::new([0; 32]));
        // When transfer 100 tokens to an account.
        let result = env.transfer(addr, 100.into());
        // Then the transfer should be successful.
        assert!(result.is_ok());
    }

    #[test]
    fn test_failing_transfer_to_account() {
        // Given a host context that fails to transfer tokens.
        let mut ctx = MockHostContext::new();
        ctx.expect_transfer()
            .returning(|_, _| Err(OdraError::ExecutionError(ExecutionError::UnwrapError)));
        let env = HostEnv::new(Rc::new(ctx));

        let addr = Address::Account(AccountHash::new([0; 32]));
        // When transfer 100 tokens to an account.
        let result = env.transfer(addr, 100.into());
        // Then the transfer should fail.
        assert_eq!(
            result.err(),
            Some(OdraError::ExecutionError(ExecutionError::UnwrapError))
        );
    }

    #[test]
    fn test_transfer_to_contract() {
        // Given a host context that successfully transfers tokens.
        let mut ctx = MockHostContext::new();
        ctx.expect_transfer().returning(|_, _| Ok(()));
        let env = HostEnv::new(Rc::new(ctx));

        let addr = Address::Contract(ContractPackageHash::new([0; 32]));
        // When transfer 100 tokens to a contract.
        let result = env.transfer(addr, 100.into());
        // Then the transfer should fail.
        assert_eq!(
            result,
            Err(OdraError::ExecutionError(
                ExecutionError::TransferToContract
            ))
        );
    }

    fn no_entry_points() -> EntryPointsCaller {
        EntryPointsCaller::new(vec![], |_, _| Ok(Bytes::default()))
    }

    #[test]
    fn upgrade_registers_a_contract_the_env_did_not_know() {
        let address = Address::Contract(ContractPackageHash::new([1; 32]));
        let mut ctx = MockHostContext::new();
        ctx.expect_upgrade_contract()
            .times(1)
            .returning(move |_, _, _, _| Ok(address));
        ctx.expect_take_child_contracts().returning(Vec::new);
        // The events emitted before belong to nobody's `last_call`.
        ctx.expect_get_events_count().times(1).returning(|_| Ok(3));
        ctx.expect_get_native_events_count()
            .times(1)
            .returning(|_| Ok(2));
        let env = HostEnv::new(Rc::new(ctx));

        env.upgrade_contract("Upgraded", address, RuntimeArgs::new(), no_entry_points())
            .unwrap();

        let contracts = env.deployed_contracts.borrow();
        let contract = contracts.get(&address).unwrap();
        assert_eq!(contract.name.as_deref(), Some("Upgraded"));
        assert_eq!(contract.current_version, 1);
        assert_eq!(contract.events_count, 3);
        assert_eq!(contract.native_events_count, 2);
    }

    #[test]
    fn register_contract_ref_skips_an_address_without_a_contract() {
        let mut ctx = MockHostContext::new();
        ctx.expect_has_contract().times(1).returning(|_| false);
        ctx.expect_register_contract().never();
        let env = HostEnv::new(Rc::new(ctx));
        let address = Address::Contract(ContractPackageHash::new([1; 32]));

        env.register_contract_ref(address, "TestRef".to_string(), no_entry_points);
        assert!(env.deployed_contracts.borrow().is_empty());
    }

    #[test]
    fn register_contract_ref_registers_once_per_name() {
        let mut ctx = MockHostContext::new();
        ctx.expect_has_contract().times(1).returning(|_| true);
        ctx.expect_register_contract()
            .with(
                predicate::always(),
                predicate::eq("TestRef".to_string()),
                predicate::always()
            )
            .times(1)
            .returning(|_, _, _| ());
        // A host ref of another type for the same address brings its code.
        ctx.expect_register_contract()
            .with(
                predicate::always(),
                predicate::eq("OtherRef".to_string()),
                predicate::always()
            )
            .times(1)
            .returning(|_, _, _| ());
        ctx.expect_get_events_count().times(1).returning(|_| Ok(3));
        ctx.expect_get_native_events_count()
            .times(1)
            .returning(|_| Ok(1));
        let env = HostEnv::new(Rc::new(ctx));
        let address = Address::Contract(ContractPackageHash::new([1; 32]));

        env.register_contract_ref(address, "TestRef".to_string(), no_entry_points);
        env.register_contract_ref(address, "TestRef".to_string(), || {
            panic!("the entry points of a registered contract are not needed")
        });
        env.register_contract_ref(address, "OtherRef".to_string(), no_entry_points);

        let contracts = env.deployed_contracts.borrow();
        let contract = contracts.get(&address).unwrap();
        assert_eq!(contract.name.as_deref(), Some("OtherRef"));
        assert_eq!(contract.events_count, 3);
        assert_eq!(contract.native_events_count, 1);
    }

    #[test]
    fn registering_a_known_contract_keeps_its_events() {
        let mut ctx = MockHostContext::new();
        ctx.expect_register_contract()
            .times(2)
            .returning(|_, _, _| ());
        // Read once, by the first registration.
        ctx.expect_get_events_count().times(1).returning(|_| Ok(2));
        ctx.expect_get_native_events_count()
            .times(1)
            .returning(|_| Ok(0));
        let env = HostEnv::new(Rc::new(ctx));
        let address = Address::Contract(ContractPackageHash::new([1; 32]));

        env.register_contract(address, "TestRef".to_string(), no_entry_points());
        env.deployed_contracts
            .borrow_mut()
            .get_mut(&address)
            .unwrap()
            .current_version = 1;
        env.register_contract(address, "TestRef".to_string(), no_entry_points());

        let contracts = env.deployed_contracts.borrow();
        let contract = contracts.get(&address).unwrap();
        assert_eq!(contract.events_count, 2);
        assert_eq!(contract.current_version, 1);
    }
}
