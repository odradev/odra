//! [`HostContext`]: what a backend implements for the host environment, the counterpart of
//! [`ContractContext`](crate::ContractContext) for the host side.

use crate::entry_point_callback::EntryPointsCaller;
use crate::gas_report::GasReport;
#[cfg(not(target_arch = "wasm32"))]
use crate::host::ThreadEnvFactory;
use crate::prelude::*;
use crate::{CallDef, ContractEnv, EventError};
use casper_types::{bytesrepr::Bytes, PublicKey, RuntimeArgs, U512};

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
    /// threads at once (livenet). `None`, the default, makes [`HostEnv::concurrently`](crate::host::HostEnv::concurrently) run its
    /// items one after another on the calling thread.
    #[cfg(not(target_arch = "wasm32"))]
    fn thread_env_factory(&self) -> Option<ThreadEnvFactory> {
        None
    }

    /// Remembers the current state of the backend, so that
    /// [`restore_snapshot`](Self::restore_snapshot) can bring it back.
    ///
    /// The default panics: the backend does not support snapshots.
    fn take_snapshot(&self) {
        panic!("Snapshots are not supported by this backend")
    }

    /// Brings the backend back to the state remembered by the last
    /// [`take_snapshot`](Self::take_snapshot).
    ///
    /// The default panics: the backend does not support snapshots.
    fn restore_snapshot(&self) {
        panic!("Snapshots are not supported by this backend")
    }

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
    ///
    /// Registering the contract again under the same name must change nothing: host refs
    /// register the contract they are made for (see [HostEnv::register_contract_ref](crate::host::HostEnv::register_contract_ref)).
    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    );

    /// Tells if `address` holds a contract on the backend, see [HostEnv::register_contract_ref](crate::host::HostEnv::register_contract_ref).
    ///
    /// The default says yes: the backend cannot tell cheaply.
    fn has_contract(&self, _address: &Address) -> bool {
        true
    }

    /// Hands over the contracts deployed by other contracts (factory children) since the last
    /// call of this function, for [HostEnv](crate::host::HostEnv) to track their events.
    ///
    /// The default returns none: the backend does not report them.
    fn take_child_contracts(&self) -> Vec<Address> {
        Vec::new()
    }

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
