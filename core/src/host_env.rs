use crate::call_result::CallResult;
use crate::call_result::ContractCallResult;
use crate::casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use crate::casper_types::{CLTyped, RuntimeArgs, U512};
use crate::consts::{ALLOW_KEY_OVERRIDE_ARG, IS_UPGRADABLE_ARG, PACKAGE_HASH_KEY_NAME_ARG};
use crate::entry_point_callback::EntryPointsCaller;
use crate::error::EventError;
use crate::host_context::HostContext;
use crate::utils::extract_event_name;
use crate::{prelude::*, OdraResult};
use crate::{Address, OdraError, VmError};
use crate::{CallDef, ContractEnv};
use casper_event_standard::EventInstance;
use casper_types::PublicKey;

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
        args.insert(IS_UPGRADABLE_ARG, false).unwrap();
        args.insert(ALLOW_KEY_OVERRIDE_ARG, true).unwrap();
        args.insert(PACKAGE_HASH_KEY_NAME_ARG, format!("{}_package_hash", name))
            .unwrap();

        let deployed_contract = backend.new_contract(name, args, entry_points_caller);

        self.deployed_contracts.borrow_mut().push(deployed_contract);
        self.events_count.borrow_mut().insert(deployed_contract, 0);
        deployed_contract
    }

    /// Registers an existing contract with the specified address and entry points caller. Similar to `new_contract`,
    /// but skips the deployment phase.
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
                    .and_then(|bytes| extract_event_name(&bytes))
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
                extract_event_name(&bytes)
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
