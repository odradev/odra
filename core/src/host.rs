//! A module that provides the interface for interacting with the host environment.

mod deployed_contracts;

use crate::address::Addressable;
use crate::gas_report::GasReport;
use crate::host::deployed_contracts::DeployedContract;
use crate::{
    call_result::CallResult, entry_point_callback::EntryPointsCaller, CallDef, ContractCallResult,
    ContractEnv, EventError
};
#[cfg(not(target_arch = "wasm32"))]
use crate::{consts, contract::OdraContract, contract_def::HasIdent};
use crate::{prelude::*, utils};
use alloc::sync::Arc;
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{Bytes, FromBytes, ToBytes},
    CLTyped, PublicKey, RuntimeArgs, U512
};
use core::time::Duration;

/// The most worker threads [`HostEnv::concurrently`] starts; each one keeps its own node
/// connection and caches.
#[cfg(not(target_arch = "wasm32"))]
const MAX_CONCURRENT_THREADS: usize = 8;

/// A host side reference to a contract.
pub trait HostRef {
    /// Creates a new host side reference to a contract.
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
    /// Returns an [EntryPointsCaller] for the given host environment.
    fn entry_points_caller(env: &HostEnv) -> EntryPointsCaller;
}

/// A type which can deploy a contract.
///
/// Before any interaction with the contract, it must be deployed, either
/// on a virtual machine or on a real blockchain.
///
/// The `Deployer` trait provides a simple way to deploy a contract.
#[cfg(not(target_arch = "wasm32"))]
pub trait Deployer<R: OdraContract>: Sized {
    /// Deploys a contract with given init args.
    ///
    /// If the `init_args` is not [NoArgs], the contract is deployed and initialized
    /// by calling the constructor. Otherwise no constructor is called.
    ///
    /// The default [OdraConfig] is used for deployment.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy(env: &HostEnv, init_args: R::InitArgs) -> R::HostRef;

    /// Tries to deploy a contract with given init args.
    ///
    /// Similar to `deploy`, but returns a result instead of panicking.
    fn try_deploy(env: &HostEnv, init_args: R::InitArgs) -> OdraResult<R::HostRef>;

    /// Deploys a contract with given init args and configuration.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy_with_cfg(env: &HostEnv, init_args: R::InitArgs, cfg: InstallConfig) -> R::HostRef;

    /// Tries to deploy a contract with given init args and configuration.
    ///
    /// Similar to `deploy_with_cfg`, but returns a result instead of panicking.
    fn try_deploy_with_cfg(
        env: &HostEnv,
        init_args: R::InitArgs,
        cfg: InstallConfig
    ) -> OdraResult<R::HostRef>;

    /// Tries to upgrade a contract with given init args.
    fn try_upgrade(
        env: &HostEnv,
        address: Address,
        init_args: R::UpgradeArgs
    ) -> OdraResult<R::HostRef>;

    /// Tries to upgrade a contract with given init args and configuration
    fn try_upgrade_with_cfg(
        env: &HostEnv,
        address: Address,
        upgrade_args: R::UpgradeArgs,
        cfg: UpgradeConfig
    ) -> OdraResult<R::HostRef>;
}

/// A type which can be used as initialization arguments for a contract.
pub trait InitArgs: Into<RuntimeArgs> {}
/// A type which can be used as upgrade arguments for a contract.
pub trait UpgradeArgs: Into<RuntimeArgs> {}

/// Default implementation of [InitArgs]. Should be used when the contract
/// does not require initialization arguments.
///
/// Precisely, it means the constructor function has not been defined,
/// or does not require any arguments.
pub struct NoArgs;

impl InitArgs for NoArgs {}

impl UpgradeArgs for NoArgs {}

impl From<NoArgs> for RuntimeArgs {
    fn from(_: NoArgs) -> Self {
        RuntimeArgs::new()
    }
}

/// A configuration for a contract.
///
/// The configuration every contract written in Odra expects.
/// Read more: [https://odra.dev/docs/backends/casper/#wasm-arguments]
#[cfg(not(target_arch = "wasm32"))]
pub struct InstallConfig {
    /// Returns the package hash of the contract.
    ///
    /// Used to set the `odra_cfg_package_hash_key_name` key at the contract initialization.
    pub package_named_key: String,
    /// Returns true if the contract should be deployed as upgradable.
    ///
    /// If true, the `odra_cfg_is_upgradable` key is set to `true` at the contract initialization.
    pub is_upgradable: bool,
    /// If true and the key `odra_cfg_package_hash_key_name` already exists, it should be overwritten.
    pub allow_key_override: bool
}

/// A configuration for upgrading contract.
///
/// The configuration every contract upgrade written in Odra expects.
/// Read more: [https://odra.dev/docs/backends/casper/#wasm-arguments]
#[cfg(not(target_arch = "wasm32"))]
pub struct UpgradeConfig {
    /// Returns the package hash of the contract.
    ///
    /// Used to set the `odra_cfg_package_hash_key_name` key at the contract initialization.
    pub package_named_key: String,
    /// Create a new upgrade group for the contract. Set it to `true` if you want to upgrade a contract
    /// Which was deployed using Odra 2.2 or earlier, or not using Odra at all.
    pub force_create_upgrade_group: bool,
    /// If true and the key `odra_cfg_package_hash_key_name` already exists, it should be overwritten.
    pub allow_key_override: bool
}

#[cfg(not(target_arch = "wasm32"))]
impl InstallConfig {
    /// Returns new InstallConfig
    pub fn new<T: HasIdent>(is_upgradable: bool, allow_key_override: bool) -> Self {
        InstallConfig {
            package_named_key: T::contract_name(),
            is_upgradable,
            allow_key_override
        }
    }

    /// Returns new InstallConfig configured for default upgradable contract
    pub fn upgradable<T: HasIdent>() -> Self {
        InstallConfig::new::<T>(true, true)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl UpgradeConfig {
    /// Returns new UpgradeConfig with default values.
    /// It is by default upgradable and allows key override.
    pub fn new<T: HasIdent>() -> Self {
        UpgradeConfig {
            package_named_key: T::contract_name(),
            force_create_upgrade_group: false,
            allow_key_override: true
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<R: OdraContract> Deployer<R> for R {
    fn deploy(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs
    ) -> <R as OdraContract>::HostRef {
        let contract_ident = R::HostRef::ident();
        match Self::try_deploy(env, init_args) {
            Ok(contract) => contract,
            Err(OdraError::ExecutionError(ExecutionError::MissingArg)) => {
                core::panic!("Invalid init args for contract {}.", contract_ident)
            }
            Err(e) => core::panic!("Contract init failed {:?}", e)
        }
    }

    fn try_deploy(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        Self::try_deploy_with_cfg(
            env,
            init_args,
            InstallConfig::new::<<R as OdraContract>::HostRef>(false, true)
        )
    }

    fn deploy_with_cfg(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs,
        cfg: InstallConfig
    ) -> <R as OdraContract>::HostRef {
        let contract_ident = R::HostRef::ident();
        match Self::try_deploy_with_cfg(env, init_args, cfg) {
            Ok(contract) => contract,
            Err(OdraError::ExecutionError(ExecutionError::MissingArg)) => {
                core::panic!("Invalid init args for contract {}.", contract_ident)
            }
            Err(e) => core::panic!("Contract init failed {:?}", e)
        }
    }

    fn try_deploy_with_cfg(
        env: &HostEnv,
        init_args: <R as OdraContract>::InitArgs,
        cfg: InstallConfig
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        let contract_ident = R::HostRef::ident();
        let caller = R::HostRef::entry_points_caller(env);

        let mut init_args = init_args.into();
        init_args.insert(consts::IS_UPGRADABLE_ARG, cfg.is_upgradable)?;
        init_args.insert(consts::IS_UPGRADE_ARG, false)?;
        init_args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, cfg.allow_key_override)?;
        init_args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            format!("{}_package_hash", cfg.package_named_key)
        )?;

        let address = env.new_contract(&contract_ident, init_args, caller)?;
        Ok(R::HostRef::new(address, env.clone()))
    }

    fn try_upgrade(
        env: &HostEnv,
        contract_to_upgrade: Address,
        upgrade_args: <R as OdraContract>::UpgradeArgs
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        Self::try_upgrade_with_cfg(
            env,
            contract_to_upgrade,
            upgrade_args,
            UpgradeConfig::new::<<R as OdraContract>::HostRef>()
        )
    }

    fn try_upgrade_with_cfg(
        env: &HostEnv,
        contract_to_upgrade: Address,
        upgrade_args: <R as OdraContract>::UpgradeArgs,
        cfg: UpgradeConfig
    ) -> OdraResult<<R as OdraContract>::HostRef> {
        let mut upgrade_args = upgrade_args.into();
        upgrade_args.insert(consts::IS_UPGRADE_ARG, true)?;
        upgrade_args.insert(
            consts::PACKAGE_HASH_TO_UPGRADE_ARG,
            contract_to_upgrade.value()
        )?;
        upgrade_args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, cfg.allow_key_override)?;
        upgrade_args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            format!("{}_package_hash", cfg.package_named_key)
        )?;
        upgrade_args.insert(consts::CREATE_UPGRADE_GROUP, cfg.force_create_upgrade_group)?;
        let contract_ident = R::HostRef::ident();
        let entry_points_caller = R::HostRef::entry_points_caller(env);

        let address = env.upgrade_contract(
            &contract_ident,
            contract_to_upgrade,
            upgrade_args,
            entry_points_caller
        )?;
        Ok(HostRef::new(address, env.clone()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: OdraContract> HostRefLoader<T::HostRef> for T {
    fn load(env: &HostEnv, address: Address) -> T::HostRef {
        let caller = T::HostRef::entry_points_caller(env);
        let contract_name = T::HostRef::ident();
        env.register_contract(address, contract_name, caller);
        T::HostRef::new(address, env.clone())
    }
}

/// Builds an independent [`HostEnv`] for a worker thread of [`HostEnv::concurrently`]; see
/// [`HostContext::thread_env_factory`].
#[derive(Clone)]
pub struct ThreadEnvFactory(Arc<dyn Fn() -> HostEnv + Send + Sync>);

impl ThreadEnvFactory {
    /// Wraps the function that builds the environment; it is called once per worker thread.
    pub fn new(build: impl Fn() -> HostEnv + Send + Sync + 'static) -> Self {
        Self(Arc::new(build))
    }

    /// Builds a new environment.
    pub fn build(&self) -> HostEnv {
        (self.0)()
    }
}

/// The `HostContext` trait defines the interface for interacting with the host environment.
#[cfg_attr(test, mockall::automock)]
pub trait HostContext {
    /// Sets the caller address for the current contract execution.
    fn set_caller(&self, caller: Address);

    /// Sets the gas limit for the current contract execution.
    fn set_gas(&self, gas: u64);

    /// Returns the caller address for the current contract execution.
    fn caller(&self) -> Address;

    /// Returns the account address at the specified index.
    fn get_account(&self, index: usize) -> Address;

    /// Returns the validator public key.
    fn get_validator(&self, index: usize) -> PublicKey;

    /// The validator at the given index will withdraw all funds and be removed from the validator set.
    fn remove_validator(&self, index: usize);

    /// Returns the CSPR balance of the specified address.
    fn balance_of(&self, address: &Address) -> U512;

    /// Advances the block time by the specified time difference.
    fn advance_block_time(&self, time_diff: u64);

    /// Advances the block time by the specified time difference and processes auctions.
    fn advance_with_auctions(&self, time_diff: u64);

    /// Time between auctions in milliseconds.
    fn auction_delay(&self) -> u64;

    /// Time for the funds to be transferred back to the delegator after undelegation in milliseconds.
    fn unbonding_delay(&self) -> u64;

    /// Returns the delegated amount for the specified delegator and validator.
    fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512;

    /// Returns the current block time.
    fn block_time(&self) -> u64;

    /// An independent environment for another thread, if the backend can be used from several
    /// threads at once (livenet). `None`, the default, makes [`HostEnv::concurrently`] run its
    /// items one after another on the calling thread.
    fn thread_env_factory(&self) -> Option<ThreadEnvFactory> {
        None
    }

    /// Remembers the current state of the backend, so that
    /// [`restore_snapshot`](Self::restore_snapshot) can bring it back.
    fn take_snapshot(&self);

    /// Brings the backend back to the state remembered by the last
    /// [`take_snapshot`](Self::take_snapshot).
    fn restore_snapshot(&self);

    /// Returns the event bytes for the specified contract address and index.
    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError>;

    /// Returns the native event bytes for the specified contract address and index.
    fn get_native_event(&self, contract_address: &Address, index: u32)
        -> Result<Bytes, EventError>;

    /// Returns the number of emitted events for the specified contract address.
    fn get_events_count(&self, contract_address: &Address) -> Result<u32, EventError>;

    /// Returns the number of emitted native events for the specified contract address.
    fn get_native_events_count(&self, contract_address: &Address) -> Result<u32, EventError>;

    /// Calls a contract at the specified address with the given call definition.
    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes>;

    /// Creates a new contract with the specified name, initialization arguments, and entry points caller.
    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address>;

    /// Upgrades an existing contract with a new one with given upgrade arguments and new entry
    /// points caller.
    fn upgrade_contract(
        &self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address>;

    /// Registers an existing contract with the specified address, name, and entry points caller.
    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    );

    /// Returns the contract environment.
    fn contract_env(&self) -> ContractEnv;

    /// Returns the gas report for the current contract execution.
    fn gas_report(&self) -> GasReport;

    /// Returns the gas cost of the last contract call.
    fn last_call_gas_cost(&self) -> u64;

    /// Signs the specified message with the given address and returns the signature.
    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes;

    /// Returns the public key associated with the specified address.
    fn public_key(&self, address: &Address) -> PublicKey;

    /// Transfers the specified amount of CSPR from the current caller to the specified address.
    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()>;

    /// Reads a raw value from the Odra storage (the `state` dictionary) of the contract
    /// at the given address.
    ///
    /// `key` is the storage key as produced by the contract environment (a hex-encoded hash).
    fn get_storage_value(&self, address: &Address, key: &[u8]) -> Option<Bytes>;

    /// Reads the raw value stored under a named key of the contract at the given address.
    fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes>;

    /// Reads the raw value stored in a named dictionary of the contract at the given address.
    fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes>;
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
    captures_events: Rc<RefCell<bool>>
}

impl HostEnv {
    /// Creates a new `HostEnv` instance with the specified backend.
    pub fn new(backend: Rc<dyn HostContext>) -> HostEnv {
        HostEnv {
            backend,
            last_call_result: RefCell::new(None).into(),
            deployed_contracts: RefCell::new(Default::default()).into(),
            captures_events: Rc::new(RefCell::new(true))
        }
    }

    /// Sets the `captures_events` flag, which determines whether events should be captured.
    pub fn set_captures_events(&self, captures: bool) {
        *self.captures_events.borrow_mut() = captures;
        if captures {
            // Initialize events for all deployed contracts if capturing is enabled
            let contract_addresses: Vec<Address> =
                self.deployed_contracts.borrow().keys().copied().collect();

            for contract_address in contract_addresses {
                self.init_events(&contract_address);
            }
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

    /// Advances the block time by `time_diff`.
    ///
    /// Block time has millisecond resolution; anything finer is truncated.
    ///
    /// ```
    /// # use core::time::Duration;
    /// # fn shift(env: &odra_core::host::HostEnv) {
    /// env.advance_block_time(Duration::from_secs(60 * 60 * 24));
    /// # }
    /// ```
    pub fn advance_block_time(&self, time_diff: Duration) {
        let backend = self.backend.as_ref();
        backend.advance_block_time(millis(time_diff))
    }

    /// Advances the block time by `time_diff` and processes auctions.
    ///
    /// Block time has millisecond resolution; anything finer is truncated.
    pub fn advance_with_auctions(&self, time_diff: Duration) {
        let backend = self.backend.as_ref();
        backend.advance_with_auctions(millis(time_diff));
    }

    /// Returns the era length.
    pub fn auction_delay(&self) -> Duration {
        let backend = self.backend.as_ref();
        Duration::from_millis(backend.auction_delay())
    }

    /// Returns the delay between unstaking and the transfer of funds back to the delegator.
    pub fn unbonding_delay(&self) -> Duration {
        let backend = self.backend.as_ref();
        Duration::from_millis(backend.unbonding_delay())
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

    /// Remembers the current state of the test VM: contract storage, CSPR balances, events and
    /// the block time.
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
        backend.take_snapshot()
    }

    /// Brings the test VM back to the state remembered by the last
    /// [`take_snapshot`](Self::take_snapshot).
    ///
    /// # Panics
    ///
    /// Panics if no snapshot has been taken.
    pub fn restore_snapshot(&self) {
        let backend = self.backend.as_ref();
        backend.restore_snapshot()
    }

    /// Runs `f` once per item and returns the results in the order of the items.
    ///
    /// On livenet the items are spread over a few worker threads, each with its own node
    /// connection and its own `HostEnv` (same caller and gas as this one), so deploys and reads
    /// that do not depend on each other overlap instead of waiting for one another. On OdraVM and
    /// CasperVM the items run one after another on this thread, with this environment.
    ///
    /// `f` gets the environment to use; it must not capture this one (the compiler enforces it:
    /// `HostEnv` cannot be sent to another thread). Deploy in the closure and hand back the
    /// address, then `load` it in the caller's environment:
    ///
    /// ```ignore
    /// env.set_gas(cspr!(450));
    /// let addresses = env.concurrently(vec![args_a, args_b, args_c], |env, args| {
    ///     Erc20::deploy(env, args).address()
    /// });
    /// let tokens: Vec<Erc20HostRef> = addresses.iter().map(|a| Erc20::load(&env, *a)).collect();
    /// let supplies = env.concurrently(addresses, |env, address| {
    ///     Erc20::load(env, address).total_supply()
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// A panic in `f` (a failed `deploy`, a revert) is propagated after the other items finish.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn concurrently<T, R, F>(&self, items: Vec<T>, f: F) -> Vec<R>
    where
        T: Send,
        R: Send,
        F: Fn(&HostEnv, T) -> R + Sync
    {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Mutex;

        let Some(factory) = self.backend.thread_env_factory() else {
            return items.into_iter().map(|item| f(self, item)).collect();
        };

        let workers = items.len().min(MAX_CONCURRENT_THREADS);
        let items: Vec<Mutex<Option<T>>> = items.into_iter().map(|i| Mutex::new(Some(i))).collect();
        let results: Vec<Mutex<Option<R>>> = items.iter().map(|_| Mutex::new(None)).collect();
        let next = AtomicUsize::new(0);

        std::thread::scope(|scope| {
            for _ in 0..workers {
                scope.spawn(|| {
                    let env = factory.build();
                    loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        let Some(slot) = items.get(index) else { break };
                        let item = slot
                            .lock()
                            .expect("item slot poisoned")
                            .take()
                            .expect("every item is taken exactly once");
                        let result = f(&env, item);
                        *results[index].lock().expect("result slot poisoned") = Some(result);
                    }
                });
            }
        });

        results
            .into_iter()
            .map(|slot| {
                slot.into_inner()
                    .expect("result slot poisoned")
                    .expect("every item produced a result")
            })
            .collect()
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
        let contract_address = backend.new_contract(name, init_args, entry_points_caller)?;

        self.deployed_contracts
            .borrow_mut()
            .insert(contract_address, DeployedContract::new(contract_address));
        // The events of `init` belong to the deploy, not to the first call after it.
        if *self.captures_events.borrow() {
            self.init_events(&contract_address);
        }
        Ok(contract_address)
    }

    /// Upgrades an existing contract with a new one with given upgrade arguments and new entry
    /// points caller.
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
        let upgraded_contract = backend.upgrade_contract(
            name,
            contract_to_upgrade,
            upgrade_args,
            entry_points_caller
        )?;
        let mut contracts = self.deployed_contracts.borrow_mut();
        let contract = contracts.get_mut(&upgraded_contract).unwrap();
        contract.current_version += 1;
        // CES events are intact, but native events are connected to a contract, not a package.
        contract.native_events_count = 0;
        Ok(upgraded_contract)
    }

    /// Registers an existing contract with the specified address, name and entry points caller.
    /// Similar to `new_contract`, but skips the deployment phase.
    pub fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        let backend = self.backend.as_ref();
        backend.register_contract(address, contract_name, entry_points_caller);
        self.deployed_contracts
            .borrow_mut()
            .insert(address, DeployedContract::new(address));
        // Events emitted before the contract was loaded belong to nobody's `last_call`.
        if *self.captures_events.borrow() {
            self.init_events(&address);
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

    /// Retrieves an event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_event<T: FromBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        index: i32
    ) -> Result<T, EventError> {
        let contract_address = addr.address();
        let backend = self.backend.as_ref();
        let events_count = self.events_count(&contract_address);
        let event_absolute_position = crate::utils::event_absolute_position(events_count, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        let bytes = backend.get_event(&contract_address, event_absolute_position)?;
        let (event, remainder) = T::from_bytes(&bytes).map_err(|_| EventError::Parsing)?;

        if remainder.is_empty() {
            Ok(event)
        } else {
            Err(EventError::Formatting)
        }
    }

    /// Retrieves a native event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_native_event<T: FromBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        index: i32
    ) -> Result<T, EventError> {
        let contract_address = addr.address();
        let backend = self.backend.as_ref();
        let events_count = self.native_events_count(&contract_address);
        let event_absolute_position = crate::utils::event_absolute_position(events_count, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        let bytes = backend.get_native_event(&contract_address, event_absolute_position)?;
        T::from_bytes(&bytes)
            .map_err(|_| EventError::Parsing)
            .map(|r| r.0)
    }

    /// Retrieves a raw event (serialized) with the specified index from the specified contract.
    pub fn get_event_bytes<T: Addressable>(
        &self,
        addr: &T,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.as_ref();
        backend.get_event(&addr.address(), index)
    }

    /// Retrieves a raw native event (serialized) with the specified index from the specified contract.
    pub fn get_native_event_bytes<T: Addressable>(
        &self,
        addr: &T,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.as_ref();
        backend.get_native_event(&addr.address(), index)
    }

    /// Returns the names of all events emitted by the specified contract.
    pub fn event_names<T: Addressable>(&self, addr: &T) -> Vec<String> {
        let events_count = self.events_count(addr);

        let backend = self.backend.as_ref();
        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(&addr.address(), event_id)
                    .and_then(|bytes| utils::extract_event_name(&bytes))
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
            })
            .collect()
    }

    /// Returns all events emitted by the specified contract.
    pub fn events<T: Addressable>(&self, addr: &T) -> Vec<Bytes> {
        let backend = self.backend.as_ref();
        let contract_address = addr.address();
        let events_count = backend
            .get_events_count(&contract_address)
            .unwrap_or_default();
        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(&contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            &contract_address, event_id, e
                        )
                    })
            })
            .collect()
    }

    /// Returns the number of events emitted by the specified contract.
    pub fn events_count<T: Addressable>(&self, addr: &T) -> u32 {
        let backend = self.backend.as_ref();
        backend
            .get_events_count(&addr.address())
            .unwrap_or_default()
    }

    /// Returns the number of native events emitted by the specified contract.
    pub fn native_events_count<T: Addressable>(&self, addr: &T) -> u32 {
        let backend = self.backend.as_ref();
        backend
            .get_native_events_count(&addr.address())
            .unwrap_or_default()
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_event<T: ToBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        event: T
    ) -> bool {
        let contract_address = addr.address();
        let events_count = self.events_count(addr);

        let event_bytes = Bytes::from(
            event
                .to_bytes()
                .unwrap_or_else(|_| panic!("Couldn't serialize event"))
        );

        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(&contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            &contract_address, event_id, e
                        )
                    })
            })
            .any(|bytes| bytes == event_bytes)
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_native_event<T: ToBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        event: T
    ) -> bool {
        let contract_address = addr.address();
        let events_count = self.native_events_count(addr);
        if events_count > 0 {
            let event_bytes = Bytes::from(
                event
                    .to_bytes()
                    .unwrap_or_else(|_| panic!("Couldn't serialize event"))
            );
            (0..events_count)
                .map(|event_id| {
                    self.get_native_event_bytes(&contract_address, event_id)
                        .unwrap_or_else(|e| {
                            panic!(
                                "Couldn't get event at address {:?} with id {}: {:?}",
                                &contract_address, event_id, e
                            )
                        })
                })
                .any(|bytes| bytes == event_bytes)
        } else {
            false
        }
    }

    /// Returns true if an event with the specified name was emitted by the specified contract.
    pub fn emitted<T: AsRef<str>, R: Addressable>(&self, addr: &R, event_name: T) -> bool {
        let events_count = self.events_count(addr);

        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(addr, event_id).unwrap_or_else(|e| {
                    panic!(
                        "Couldn't get event at address {:?} with id {}: {:?}",
                        addr.address(),
                        event_id,
                        e
                    )
                })
            })
            .any(|bytes| {
                utils::extract_event_name(&bytes)
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
                    .as_str()
                    == event_name.as_ref()
            })
    }
    /// Returns true if a native event with the specified name was emitted by the specified contract.
    pub fn emitted_native<T: AsRef<str>, R: Addressable>(&self, addr: &R, event_name: T) -> bool {
        let events_count = self.native_events_count(addr);

        (0..events_count)
            .map(|event_id| {
                self.get_native_event_bytes(addr, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            addr.address(),
                            event_id,
                            e
                        )
                    })
            })
            .any(|bytes| {
                utils::extract_event_name(&bytes)
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
                    .as_str()
                    == event_name.as_ref()
            })
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

    fn last_events(&self, contract: &mut DeployedContract) -> Vec<Bytes> {
        let old_count = contract.events_count;
        let new_count = self.events_count(&contract.address);
        let mut events = vec![];
        for count in old_count..new_count {
            let event = self.get_event_bytes(&contract.address, count).unwrap();
            events.push(event);
        }

        contract.events_count = new_count;
        events
    }

    fn last_native_events(&self, contract: &mut DeployedContract) -> Vec<Bytes> {
        let old_count = contract.native_events_count;
        let new_count = self.native_events_count(&contract.address);
        let mut events = vec![];
        for count in old_count..new_count {
            let event = self
                .get_native_event_bytes(&contract.address, count)
                .unwrap();
            events.push(event);
        }

        contract.native_events_count = new_count;
        events
    }

    fn init_events(&self, contract_address: &Address) {
        // First, check if initialization is needed and get event counts
        let needs_init = {
            let contracts = self.deployed_contracts.borrow();
            contracts
                .get(contract_address)
                .map(|contract| !contract.events_initialized)
                .unwrap_or(false)
        };

        if needs_init {
            // Get event counts while not holding any borrows
            let events_count = self.events_count(contract_address);
            let native_events_count = self.native_events_count(contract_address);

            // Now update the contract
            let mut contracts = self.deployed_contracts.borrow_mut();
            let contract = contracts.get_mut(contract_address).unwrap();
            contract.events_count = events_count;
            contract.native_events_count = native_events_count;
            contract.events_initialized = true;
        }
    }
}

/// Block time is kept in milliseconds by every backend.
fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod test {
    use core::fmt::Debug;

    use super::*;
    use casper_event_standard::Event;
    use casper_types::account::AccountHash;
    use casper_types::contracts::ContractPackageHash;
    use mockall::{mock, predicate};
    use std::sync::Mutex;

    static IDENT_MTX: Mutex<()> = Mutex::new(());
    static EPC_MTX: Mutex<()> = Mutex::new(());

    #[derive(Debug, Event, PartialEq)]
    struct TestEv {}

    mock! {
        TestRef {}
        impl HasIdent for TestRef {
            fn ident() -> String;
        }
        impl EntryPointsCallerProvider for TestRef {
            fn entry_points_caller(env: &HostEnv) -> EntryPointsCaller;
        }
        impl HostRef for TestRef {
            fn new(address: Address, env: HostEnv) -> Self;
            fn with_tokens(&self, tokens: U512) -> Self;
            fn contract_address(&self) -> Address;
            fn env(&self) -> &HostEnv;
            fn get_event<T>(&self, index: i32) -> Result<T, EventError> where T: FromBytes + EventInstance + 'static;
            fn last_call(&self) -> ContractCallResult;
        }
    }

    impl crate::ContractRef for MockTestRef {
        fn new(_env: Rc<ContractEnv>, _address: Address) -> Self {
            unimplemented!()
        }
        fn address(&self) -> &Address {
            unimplemented!()
        }

        fn with_tokens(&self, _tokens: U512) -> Self {
            unimplemented!()
        }
    }

    impl OdraContract for MockTestRef {
        type HostRef = MockTestRef;

        type ContractRef = MockTestRef;

        type InitArgs = NoArgs;

        type UpgradeArgs = NoArgs;
    }

    mock! {
        Ev {}
        impl Into<RuntimeArgs> for Ev {
            fn into(self) -> RuntimeArgs;
        }
    }

    #[test]
    fn concurrently_runs_on_the_calling_thread_without_a_factory() {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| None);
        let env = HostEnv::new(Rc::new(ctx));
        let main_thread = std::thread::current().id();

        let results = env.concurrently((0..5).collect(), |_, i: u32| {
            (i * 2, std::thread::current().id())
        });

        assert_eq!(
            results,
            (0..5).map(|i| (i * 2, main_thread)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn concurrently_spreads_items_over_worker_threads_and_keeps_the_order() {
        let mut ctx = MockHostContext::new();
        ctx.expect_thread_env_factory().returning(|| {
            Some(ThreadEnvFactory::new(|| {
                let mut ctx = MockHostContext::new();
                ctx.expect_get_account()
                    .returning(|_| Address::Account(AccountHash::new([7; 32])));
                HostEnv::new(Rc::new(ctx))
            }))
        });
        let env = HostEnv::new(Rc::new(ctx));
        let main_thread = std::thread::current().id();

        let results = env.concurrently((0..50u32).collect(), |env, i| {
            // The worker's own environment is usable, and the work really runs elsewhere.
            assert_eq!(
                env.get_account(0),
                Address::Account(AccountHash::new([7; 32]))
            );
            std::thread::sleep(core::time::Duration::from_millis(1));
            (i * 2, std::thread::current().id())
        });

        let values: Vec<u32> = results.iter().map(|(v, _)| *v).collect();
        assert_eq!(values, (0..50).map(|i| i * 2).collect::<Vec<_>>());
        let threads: std::collections::HashSet<_> = results.iter().map(|(_, t)| *t).collect();
        assert!(threads.len() > 1, "expected several worker threads");
        assert!(!threads.contains(&main_thread));
    }

    #[test]
    fn test_deploy_with_default_args() {
        // MockTestRef::ident() and  MockTestRef::entry_points_caller() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _i = IDENT_MTX.lock();
        let _e = EPC_MTX.lock();

        // stubs
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|h| EntryPointsCaller::new(h.clone(), vec![], |_, _| Ok(Bytes::default())));

        // check if TestRef::new() is called exactly once
        let instance_ctx = MockTestRef::new_context();
        instance_ctx
            .expect()
            .times(1)
            .returning(|_, _| MockTestRef::default());

        let mut ctx = MockHostContext::new();
        ctx.expect_new_contract()
            .returning(|_, _, _| Ok(Address::Account(AccountHash::new([0; 32]))));
        // The event baseline of the new contract is read right after the deploy.
        ctx.expect_get_events_count().returning(|_| Ok(0));
        ctx.expect_get_native_events_count().returning(|_| Ok(0));
        let env = HostEnv::new(Rc::new(ctx));
        MockTestRef::deploy(&env, NoArgs);
    }

    #[test]
    fn test_load_ref() {
        // MockTestRef::ident() and MockTestRef::entry_points_caller() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _e = EPC_MTX.lock();
        let _i = IDENT_MTX.lock();

        // stubs
        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|h| EntryPointsCaller::new(h.clone(), vec![], |_, _| Ok(Bytes::default())));
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let mut ctx = MockHostContext::new();
        ctx.expect_register_contract().returning(|_, _, _| ());
        ctx.expect_get_events_count().returning(|_| Ok(0));
        ctx.expect_get_native_events_count().returning(|_| Ok(0));

        // check if TestRef::new() is called exactly once
        let instance_ctx = MockTestRef::new_context();
        instance_ctx
            .expect()
            .times(1)
            .returning(|_, _| MockTestRef::default());

        let env = HostEnv::new(Rc::new(ctx));
        let address = Address::Account(AccountHash::new([0; 32]));
        MockTestRef::load(&env, address);
    }

    #[test]
    fn test_host_env() {
        let mut ctx = MockHostContext::new();
        ctx.expect_new_contract()
            .returning(|_, _, _| Ok(Address::Account(AccountHash::new([0; 32]))));
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

    #[test]
    fn test_get_event() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(ctx));

        assert_eq!(env.get_event(&addr, 1), Ok(TestEv {}));
        assert_eq!(env.get_event(&addr, -1), Ok(TestEv {}));
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, 0),
            Err(EventError::Parsing)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, -2),
            Err(EventError::Parsing)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, 2),
            Err(EventError::IndexOutOfBounds)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, -3),
            Err(EventError::IndexOutOfBounds)
        );
    }

    #[test]
    fn test_events_works() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(vec![1, 0, 1].into()));

        let env = HostEnv::new(Rc::new(ctx));

        assert_eq!(
            env.events(&addr),
            vec![vec![1].into(), vec![1, 0, 1].into()]
        );
    }

    #[test]
    #[should_panic(
        expected = "Couldn't get event at address Account(AccountHash(0000000000000000000000000000000000000000000000000000000000000000)) with id 0: CouldntExtractEventData"
    )]
    fn test_events_fails() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 panics
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Err(EventError::CouldntExtractEventData));

        let env = HostEnv::new(Rc::new(ctx));

        env.events(&addr);
    }

    #[test]
    fn test_emitted() {
        let addr = Address::Account(AccountHash::new([0; 32]));
        let mut ctx = MockHostContext::new();

        ctx.expect_get_events_count().returning(|_| Ok(1));
        ctx.expect_get_event()
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(ctx));
        assert!(env.emitted(&addr, "TestEv"));
        assert!(!env.emitted(&addr, "AnotherEvent"));
    }
}
