//! A module that provides the interface for interacting with the host environment.

use crate::address::Addressable;
use crate::gas_report::GasReport;
use crate::{
    call_result::CallResult, contract_def::HasIdent, entry_point_callback::EntryPointsCaller,
    Address, CallDef, ContractCallResult, ContractEnv, EventError, OdraError, OdraResult, VmError
};
use crate::{consts, prelude::*, utils, ExecutionError};
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{Bytes, FromBytes, ToBytes},
    CLTyped, PublicKey, RuntimeArgs, U512
};

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
    fn address(&self) -> &Address;
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
    fn address(&self) -> &Address {
        HostRef::address(self)
    }
}

/// Trait for loading a contract from the host environment.
///
/// Similar to [Deployer], but does not deploy a new contract, but loads an existing one.
pub trait HostRefLoader {
    /// Loads an existing contract from the host environment.
    fn load(env: &HostEnv, address: Address) -> Self;
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
pub trait Deployer: Sized {
    /// Deploys a contract with given init args.
    ///
    /// If the init args are provided, the contract is deployed and initialized
    /// by calling the constructor. If the init args are not provided, the contract
    /// is deployed without initialization.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> Self;

    /// Tries to deploy a contract with given init args.
    ///
    /// Similar to `deploy`, but returns a result instead of panicking.
    fn try_deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> OdraResult<Self>;
}

/// A type which can be used as initialization arguments for a contract.
pub trait InitArgs: Into<RuntimeArgs> {
    /// Validates the args are used to initialized the right contact.
    ///
    /// If the `expected_ident` does not match the contract ident, the method returns `false`.
    fn validate(expected_ident: &str) -> bool;
}

/// Default implementation of [InitArgs]. Should be used when the contract
/// does not require initialization arguments.
///
/// Precisely, it means the constructor function has not been defined,
/// or does not require any arguments.
pub struct NoArgs;

impl InitArgs for NoArgs {
    fn validate(_expected_ident: &str) -> bool {
        true
    }
}

impl From<NoArgs> for RuntimeArgs {
    fn from(_: NoArgs) -> Self {
        RuntimeArgs::new()
    }
}

impl<R: HostRef + EntryPointsCallerProvider + HasIdent> Deployer for R {
    fn deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> Self {
        let contract_ident = R::ident();
        match Self::try_deploy(env, init_args) {
            Ok(contract) => contract,
            Err(OdraError::ExecutionError(ExecutionError::MissingArg)) => {
                core::panic!("Invalid init args for contract {}.", contract_ident)
            }
            Err(e) => core::panic!("Contract init failed {:?}", e)
        }
    }

    fn try_deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> OdraResult<Self> {
        let contract_ident = R::ident();
        if !T::validate(&contract_ident) {
            return Err(OdraError::ExecutionError(ExecutionError::MissingArg));
        }

        let caller = R::entry_points_caller(env);
        let address = env.new_contract(&contract_ident, init_args.into(), caller)?;
        Ok(R::new(address, env.clone()))
    }
}

impl<T: EntryPointsCallerProvider + HostRef + HasIdent> HostRefLoader for T {
    fn load(env: &HostEnv, address: Address) -> Self {
        let caller = T::entry_points_caller(env);
        let contract_name = T::ident();
        env.register_contract(address, contract_name, caller);
        T::new(address, env.clone())
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

    /// Returns the CSPR balance of the specified address.
    fn balance_of(&self, address: &Address) -> U512;

    /// Advances the block time by the specified time difference.
    fn advance_block_time(&self, time_diff: u64);

    /// Returns the current block time.
    fn block_time(&self) -> u64;

    /// Returns the event bytes for the specified contract address and index.
    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError>;

    /// Returns the number of emitted events for the specified contract address.
    fn get_events_count(&self, contract_address: &Address) -> u32;

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
}

/// Represents the host environment for executing smart contracts.
///
/// It provides methods for interacting with the underlying host context and managing
/// the execution of contracts.
#[derive(Clone)]
pub struct HostEnv {
    backend: Rc<RefCell<dyn HostContext>>,
    last_call_result: Rc<RefCell<Option<CallResult>>>,
    deployed_contracts: Rc<RefCell<Vec<Address>>>,
    events_count: Rc<RefCell<BTreeMap<Address, u32>>> // contract_address -> events_count
}

impl HostEnv {
    /// Creates a new `HostEnv` instance with the specified backend.
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv {
            backend,
            last_call_result: RefCell::new(None).into(),
            deployed_contracts: RefCell::new(vec![]).into(),
            events_count: Rc::new(RefCell::new(Default::default()))
        }
    }

    /// Returns the account address at the specified index.
    pub fn get_account(&self, index: usize) -> Address {
        let backend = self.backend.borrow();
        backend.get_account(index)
    }

    /// Sets the caller address for the current contract execution.
    pub fn set_caller(&self, address: Address) {
        if address.is_contract() {
            panic!("Caller cannot be a contract: {:?}", address)
        }
        let backend = self.backend.borrow();
        backend.set_caller(address)
    }

    /// Advances the block time by the specified time difference.
    pub fn advance_block_time(&self, time_diff: u64) {
        let backend = self.backend.borrow();
        backend.advance_block_time(time_diff)
    }

    /// Returns the current block time.
    pub fn block_time(&self) -> u64 {
        let backend = self.backend.borrow();
        backend.block_time()
    }

    /// Registers a new contract with the specified name, initialization arguments, and entry points caller.
    pub fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let backend = self.backend.borrow();

        let mut init_args = init_args;
        init_args.insert(consts::IS_UPGRADABLE_ARG, false).unwrap();
        init_args
            .insert(consts::ALLOW_KEY_OVERRIDE_ARG, true)
            .unwrap();
        init_args
            .insert(
                consts::PACKAGE_HASH_KEY_NAME_ARG,
                format!("{}_package_hash", name)
            )
            .unwrap();

        let deployed_contract = backend.new_contract(name, init_args, entry_points_caller)?;

        self.deployed_contracts.borrow_mut().push(deployed_contract);
        self.events_count.borrow_mut().insert(deployed_contract, 0);
        Ok(deployed_contract)
    }

    /// Registers an existing contract with the specified address, name and entry points caller.
    /// Similar to `new_contract`, but skips the deployment phase.
    pub fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        let backend = self.backend.borrow();
        backend.register_contract(address, contract_name, entry_points_caller);
        self.deployed_contracts.borrow_mut().push(address);
        self.events_count
            .borrow_mut()
            .insert(address, backend.get_events_count(&address));
    }

    /// Calls a contract at the specified address with the given call definition.
    pub fn call_contract<T: FromBytes + CLTyped>(
        &self,
        address: Address,
        call_def: CallDef
    ) -> OdraResult<T> {
        let backend = self.backend.borrow();
        let use_proxy = T::cl_type() != <()>::cl_type() || !call_def.amount().is_zero();
        let call_result = backend.call_contract(&address, call_def, use_proxy);

        let mut events_map: BTreeMap<Address, Vec<Bytes>> = BTreeMap::new();
        let mut binding = self.events_count.borrow_mut();

        // Go through all contracts and collect their events
        self.deployed_contracts
            .borrow()
            .iter()
            .for_each(|contract_address| {
                let events_count = binding.get_mut(contract_address).unwrap();
                let old_events_last_id = *events_count;
                let new_events_count = backend.get_events_count(contract_address);
                let mut events = vec![];
                for event_id in old_events_last_id..new_events_count {
                    let event = backend.get_event(contract_address, event_id).unwrap();
                    events.push(event);
                }

                events_map.insert(*contract_address, events);

                *events_count = new_events_count;
            });

        let last_call_gas_cost = backend.last_call_gas_cost();

        self.last_call_result.replace(Some(CallResult::new(
            address,
            backend.caller(),
            last_call_gas_cost,
            call_result.clone(),
            events_map
        )));

        call_result.map(|bytes| {
            T::from_bytes(&bytes)
                .map(|(obj, _)| obj)
                .map_err(|_| OdraError::VmError(VmError::Deserialization))
        })?
    }

    /// Returns the gas cost of the last contract call.
    pub fn contract_env(&self) -> ContractEnv {
        self.backend.borrow().contract_env()
    }

    /// Prints the gas report for the current contract execution.
    pub fn gas_report(&self) -> GasReport {
        self.backend.borrow().gas_report().clone()
    }

    /// Returns the CSPR balance of the specified address.
    pub fn balance_of<T: Addressable>(&self, address: &T) -> U512 {
        let backend = self.backend.borrow();
        backend.balance_of(address.address())
    }

    /// Retrieves an event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_event<T: FromBytes + EventInstance, R: Addressable>(
        &self,
        contract_address: &R,
        index: i32
    ) -> Result<T, EventError> {
        let contract_address = contract_address.address();
        let backend = self.backend.borrow();
        let events_count = self.events_count(contract_address);
        let event_absolute_position = crate::utils::event_absolute_position(events_count, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        let bytes = backend.get_event(contract_address, event_absolute_position)?;
        T::from_bytes(&bytes)
            .map_err(|_| EventError::Parsing)
            .map(|r| r.0)
    }

    /// Retrieves a raw event (serialized) with the specified index from the specified contract.
    pub fn get_event_bytes<T: Addressable>(
        &self,
        contract_address: &T,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.borrow();
        backend.get_event(contract_address.address(), index)
    }

    /// Returns the names of all events emitted by the specified contract.
    pub fn event_names<T: Addressable>(&self, contract_address: &T) -> Vec<String> {
        let backend = self.backend.borrow();
        let events_count = backend.get_events_count(contract_address.address());

        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(contract_address.address(), event_id)
                    .and_then(|bytes| utils::extract_event_name(&bytes))
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
            })
            .collect()
    }

    /// Returns all events emitted by the specified contract.
    pub fn events<T: Addressable>(&self, contract_address: &T) -> Vec<Bytes> {
        let backend = self.backend.borrow();
        let contract_address = contract_address.address();
        let events_count = backend.get_events_count(contract_address);
        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(contract_address, event_id)
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
    pub fn events_count<T: Addressable>(&self, contract_address: &T) -> u32 {
        let backend = self.backend.borrow();
        backend.get_events_count(contract_address.address())
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_event<T: ToBytes + EventInstance, R: Addressable>(
        &self,
        contract_address: &R,
        event: &T
    ) -> bool {
        let contract_address = contract_address.address();
        let events_count = self.events_count(contract_address);
        let event_bytes = Bytes::from(
            event
                .to_bytes()
                .unwrap_or_else(|_| panic!("Couldn't serialize event"))
        );
        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            &contract_address, event_id, e
                        )
                    })
            })
            .any(|bytes| bytes == event_bytes)
    }

    /// Returns true if an event with the specified name was emitted by the specified contract.
    pub fn emitted<T: AsRef<str>, R: Addressable>(
        &self,
        contract_address: &R,
        event_name: T
    ) -> bool {
        let events_count = self.events_count(contract_address);
        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            contract_address.address(),
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
        let backend = self.backend.borrow();
        backend.sign_message(message, address)
    }

    /// Returns the public key associated with the specified address.
    pub fn public_key(&self, address: &Address) -> PublicKey {
        let backend = self.backend.borrow();
        backend.public_key(address)
    }

    /// Returns the caller address for the current contract execution.
    pub fn caller(&self) -> Address {
        let backend = self.backend.borrow();
        backend.caller()
    }

    /// Sets the gas limit for the current contract execution.
    pub fn set_gas(&self, gas: u64) {
        let backend = self.backend.borrow();
        backend.set_gas(gas)
    }

    /// Transfers the specified amount of CSPR from the current caller to the specified address.
    pub fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        if to.is_contract() {
            return Err(OdraError::ExecutionError(
                ExecutionError::TransferToContract
            ));
        }
        let backend = self.backend.borrow();
        backend.transfer(to, amount)
    }
}

#[cfg(test)]
mod test {
    use core::fmt::Debug;

    use super::*;
    use casper_event_standard::Event;
    use casper_types::account::AccountHash;
    use casper_types::PackageHash;
    use mockall::{mock, predicate};
    use std::sync::Mutex;

    static IDENT_MTX: Mutex<()> = Mutex::new(());
    static EPC_MTX: Mutex<()> = Mutex::new(());
    static VALIDATE_MTX: Mutex<()> = Mutex::new(());

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
            fn address(&self) -> &Address;
            fn env(&self) -> &HostEnv;
            fn get_event<T>(&self, index: i32) -> Result<T, EventError> where T: FromBytes + EventInstance + 'static;
            fn last_call(&self) -> ContractCallResult;
        }
    }

    mock! {
        Ev {}
        impl InitArgs for Ev {
            fn validate(expected_ident: &str) -> bool;
        }
        impl Into<RuntimeArgs> for Ev {
            fn into(self) -> RuntimeArgs;
        }
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
        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));
        <MockTestRef as Deployer>::deploy(&env, NoArgs);
    }

    #[test]
    #[should_panic(expected = "Invalid init args for contract TestRef.")]
    fn test_deploy_with_invalid_args() {
        // MockTestRef::ident() and  MockEv::validate() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _i = IDENT_MTX.lock();
        let _v = VALIDATE_MTX.lock();

        // stubs
        let args_ctx = MockEv::validate_context();
        args_ctx.expect().returning(|_| false);
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let env = HostEnv::new(Rc::new(RefCell::new(MockHostContext::new())));
        let args = MockEv::new();
        MockTestRef::deploy(&env, args);
    }

    #[test]
    fn test_load_ref() {
        // MockTestRef::ident(), MockEv::validate(), MockTestRef::entry_points_caller() are static and can't be safely used
        // from multiple tests at the same time. Should be to protected with a Mutex. Each function has
        // a separate Mutex.
        // https://github.com/asomers/mockall/blob/master/mockall/tests/mock_struct_with_static_method.rs
        let _e = EPC_MTX.lock();
        let _i = IDENT_MTX.lock();
        let _v = VALIDATE_MTX.lock();

        // stubs
        let args_ctx = MockEv::validate_context();
        args_ctx.expect().returning(|_| true);
        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|h| EntryPointsCaller::new(h.clone(), vec![], |_, _| Ok(Bytes::default())));
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let mut ctx = MockHostContext::new();
        ctx.expect_register_contract().returning(|_, _, _| ());
        ctx.expect_get_events_count().returning(|_| 0);

        // check if TestRef::new() is called exactly once
        let instance_ctx = MockTestRef::new_context();
        instance_ctx
            .expect()
            .times(1)
            .returning(|_, _| MockTestRef::default());

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));
        let address = Address::Account(AccountHash::new([0; 32]));
        <MockTestRef as HostRefLoader>::load(&env, address);
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

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

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
        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

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
        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

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
        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

        let addr = Address::Contract(PackageHash::new([0; 32]));
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
        ctx.expect_get_events_count().returning(|_| 2);
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

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
        ctx.expect_get_events_count().returning(|_| 2);
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(vec![1, 0, 1].into()));

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

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
        ctx.expect_get_events_count().returning(|_| 2);
        // get_event() at index 0 panics
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Err(EventError::CouldntExtractEventData));

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

        env.events(&addr);
    }

    #[test]
    fn test_emitted() {
        let addr = Address::Account(AccountHash::new([0; 32]));
        let mut ctx = MockHostContext::new();

        ctx.expect_get_events_count().returning(|_| 1);
        ctx.expect_get_event()
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));
        assert!(env.emitted(&addr, "TestEv"));
        assert!(!env.emitted(&addr, "AnotherEvent"));
    }
}
