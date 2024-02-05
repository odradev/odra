//! A module that provides the interface for interacting with the host environment.

use crate::{
    call_result::CallResult, contract_def::HasIdent, entry_point_callback::EntryPointsCaller,
    Address, CallDef, ContractCallResult, ContractEnv, EventError, OdraError, OdraResult, VmError
};
use crate::{consts, prelude::*, utils};
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
    fn get_event<T: 'static>(&self, index: i32) -> Result<T, EventError>
    where
        T: FromBytes + EventInstance;
    /// Returns a detailed information about the last call of the contract.
    fn last_call(&self) -> ContractCallResult;
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
pub trait Deployer {
    /// Deploys a contract with given init args.
    ///
    /// If the init args are provided, the contract is deployed and initialized
    /// by calling the constructor. If the init args are not provided, the contract
    /// is deployed without initialization.
    ///
    /// Returns a host reference to the deployed contract.
    fn deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> Self;
}

/// A type which can be used as initialization arguments for a contract.
#[cfg_attr(test, mockall::automock)]
pub trait InitArgs {
    /// Validates the args are used to initialized the right contact.
    ///
    /// If the `expected_ident` does not match the contract ident, the method returns `false`.
    fn validate(expected_ident: &str) -> bool;
    /// Converts the init args into a [RuntimeArgs] instance.
    fn into_runtime_args(self) -> Option<RuntimeArgs>;
}

/// Default implementation of [InitArgs]. Should be used when the contract
/// has the constructor but does not require any initialization arguments.
pub struct NoArgs;

impl InitArgs for NoArgs {
    fn validate(_expected_ident: &str) -> bool {
        true
    }

    fn into_runtime_args(self) -> Option<RuntimeArgs> {
        Some(RuntimeArgs::new())
    }
}

/// An implementation of [InitArgs]. Should be used when the contract
/// does not have the constructor.
pub struct NoInit;

impl InitArgs for NoInit {
    fn validate(_expected_ident: &str) -> bool {
        true
    }

    fn into_runtime_args(self) -> Option<RuntimeArgs> {
        None
    }
}

impl<R: HostRef + EntryPointsCallerProvider + HasIdent> Deployer for R {
    fn deploy<T: InitArgs>(env: &HostEnv, init_args: T) -> Self {
        let contract_ident = R::ident();
        if !T::validate(&contract_ident) {
            core::panic!("Invalid init args for contract {}.", contract_ident);
        }

        let caller = R::entry_points_caller(env);
        let address = env.new_contract(&contract_ident, init_args.into_runtime_args(), caller);
        R::new(address, env.clone())
    }
}

impl<T: EntryPointsCallerProvider + HostRef> HostRefLoader for T {
    fn load(env: &HostEnv, address: Address) -> Self {
        let caller = T::entry_points_caller(env);
        env.register_contract(address, caller);
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
    ) -> Address;

    /// Registers an existing contract with the specified address and entry points caller.
    fn register_contract(&self, address: Address, entry_points_caller: EntryPointsCaller);

    /// Returns the contract environment.
    fn contract_env(&self) -> ContractEnv;

    /// Prints the gas report for the current contract execution.
    fn print_gas_report(&self);

    /// Returns the gas cost of the last contract call.
    fn last_call_gas_cost(&self) -> u64;

    /// Signs the specified message with the given address and returns the signature.
    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes;

    /// Returns the public key associated with the specified address.
    fn public_key(&self, address: &Address) -> PublicKey;
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
        let backend = self.backend.borrow();
        backend.set_caller(address)
    }

    /// Advances the block time by the specified time difference.
    pub fn advance_block_time(&self, time_diff: u64) {
        let backend = self.backend.borrow();
        backend.advance_block_time(time_diff)
    }

    /// Registers a new contract with the specified name, initialization arguments, and entry points caller.
    pub fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: EntryPointsCaller
    ) -> Address {
        let backend = self.backend.borrow();

        let mut args = match init_args {
            None => RuntimeArgs::new(),
            Some(args) => args
        };
        args.insert(consts::IS_UPGRADABLE_ARG, false).unwrap();
        args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true).unwrap();
        args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            format!("{}_package_hash", name)
        )
        .unwrap();

        let deployed_contract = backend.new_contract(name, args, entry_points_caller);

        self.deployed_contracts.borrow_mut().push(deployed_contract);
        self.events_count.borrow_mut().insert(deployed_contract, 0);
        deployed_contract
    }

    /// Registers an existing contract with the specified address and entry points caller.
    /// Similar to `new_contract`, but skips the deployment phase.
    pub fn register_contract(&self, address: Address, entry_points_caller: EntryPointsCaller) {
        let backend = self.backend.borrow();
        backend.register_contract(address, entry_points_caller);
        self.deployed_contracts.borrow_mut().push(address);
        self.events_count
            .borrow_mut()
            .insert(address, self.events_count(&address));
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

        self.last_call_result.replace(Some(CallResult::new(
            address,
            backend.caller(),
            backend.last_call_gas_cost(),
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
    pub fn print_gas_report(&self) {
        let backend = self.backend.borrow();
        backend.print_gas_report()
    }

    /// Returns the CSPR balance of the specified address.
    pub fn balance_of(&self, address: &Address) -> U512 {
        let backend = self.backend.borrow();
        backend.balance_of(address)
    }

    /// Retrieves an event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_event<T: FromBytes + EventInstance>(
        &self,
        contract_address: &Address,
        index: i32
    ) -> Result<T, EventError> {
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
    pub fn get_event_bytes(
        &self,
        contract_address: &Address,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.borrow();
        backend.get_event(contract_address, index)
    }

    /// Returns the names of all events emitted by the specified contract.
    pub fn event_names(&self, contract_address: &Address) -> Vec<String> {
        let backend = self.backend.borrow();
        let events_count = backend.get_events_count(contract_address);

        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(contract_address, event_id)
                    .and_then(|bytes| utils::extract_event_name(&bytes))
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
            })
            .collect()
    }

    /// Returns all events emitted by the specified contract.
    pub fn events(&self, contract_address: &Address) -> Vec<Bytes> {
        let backend = self.backend.borrow();
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
    pub fn events_count(&self, contract_address: &Address) -> u32 {
        let backend = self.backend.borrow();
        backend.get_events_count(contract_address)
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_event<T: ToBytes + EventInstance>(
        &self,
        contract_address: &Address,
        event: &T
    ) -> bool {
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
    pub fn emitted<T: AsRef<str>>(&self, contract_address: &Address, event_name: T) -> bool {
        let events_count = self.events_count(contract_address);
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
}

#[cfg(test)]
mod test {
    use core::fmt::Debug;

    use super::*;
    use casper_event_standard::Event;
    use casper_types::account::AccountHash;
    use mockall::{mock, predicate};

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
            fn get_event<T: 'static>(&self, index: i32) -> Result<T, EventError> where T: FromBytes + EventInstance;
            fn last_call(&self) -> ContractCallResult;
        }
    }

    #[test]
    fn test_deploy_with_default_args() {
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
            .returning(|_, _, _| Address::Account(AccountHash::new([0; 32])));
        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));
        <MockTestRef as Deployer>::deploy(&env, NoArgs);
    }

    #[test]
    #[should_panic(expected = "Invalid init args for contract TestRef.")]
    fn test_deploy_with_invalid_args() {
        // stubs
        let args_ctx = MockInitArgs::validate_context();
        args_ctx.expect().returning(|_| false);
        let indent_ctx = MockTestRef::ident_context();
        indent_ctx.expect().returning(|| "TestRef".to_string());

        let env = HostEnv::new(Rc::new(RefCell::new(MockHostContext::new())));
        let args = MockInitArgs::new();
        MockTestRef::deploy(&env, args);
    }

    #[test]
    fn test_load_ref() {
        // stubs
        let args_ctx = MockInitArgs::validate_context();
        args_ctx.expect().returning(|_| true);
        let epc_ctx = MockTestRef::entry_points_caller_context();
        epc_ctx
            .expect()
            .returning(|h| EntryPointsCaller::new(h.clone(), vec![], |_, _| Ok(Bytes::default())));

        let mut ctx = MockHostContext::new();
        ctx.expect_register_contract().returning(|_, _| ());
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
            .returning(|_, _, _| Address::Account(AccountHash::new([0; 32])));
        ctx.expect_caller()
            .returning(|| Address::Account(AccountHash::new([2; 32])))
            .times(1);
        ctx.expect_print_gas_report().returning(|| ()).times(1);
        ctx.expect_set_gas().returning(|_| ()).times(1);

        let env = HostEnv::new(Rc::new(RefCell::new(ctx)));

        assert_eq!(env.caller(), Address::Account(AccountHash::new([2; 32])));
        // should call the `HostContext`
        env.print_gas_report();
        env.set_gas(1_000u64)
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

        assert_eq!(env.get_event::<TestEv>(&addr, 1), Ok(TestEv {}));
        assert_eq!(env.get_event::<TestEv>(&addr, -1), Ok(TestEv {}));
        assert_eq!(env.get_event::<TestEv>(&addr, 0), Err(EventError::Parsing));
        assert_eq!(env.get_event::<TestEv>(&addr, -2), Err(EventError::Parsing));
        assert_eq!(
            env.get_event::<TestEv>(&addr, 2),
            Err(EventError::IndexOutOfBounds)
        );
        assert_eq!(
            env.get_event::<TestEv>(&addr, -3),
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
}
