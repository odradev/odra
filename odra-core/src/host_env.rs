use crate::call_result::CallResult;
use crate::entry_point_callback::EntryPointsCaller;
use crate::event::EventError;
use crate::host_context::HostContext;
use crate::prelude::*;
use crate::{CallDef, ContractEnv};
use alloc::collections::BTreeMap;
use casper_event_standard::EventInstance;
use odra_types::{Address, OdraError, VmError, U512};
use odra_types::{Bytes, RuntimeArgs, ToBytes};
use odra_types::{CLTyped, FromBytes};

#[derive(Clone)]
pub struct HostEnv {
    backend: Rc<RefCell<dyn HostContext>>,
    last_call_result: Rc<RefCell<Option<CallResult>>>,
    deployed_contracts: Rc<RefCell<Vec<Address>>>,
    events_count: Rc<RefCell<BTreeMap<Address, u32>>> // contract_address -> events_count
}

impl HostEnv {
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv {
            backend,
            last_call_result: RefCell::new(None).into(),
            deployed_contracts: RefCell::new(vec![]).into(),
            events_count: Rc::new(RefCell::new(Default::default()))
        }
    }

    pub fn get_account(&self, index: usize) -> Address {
        let backend = self.backend.borrow();
        backend.get_account(index)
    }

    pub fn set_caller(&self, address: Address) {
        let backend = self.backend.borrow();
        backend.set_caller(address)
    }

    pub fn advance_block_time(&self, time_diff: u64) {
        let backend = self.backend.borrow();
        backend.advance_block_time(time_diff)
    }

    pub fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        let backend = self.backend.borrow();
        let deployed_contract = backend.new_contract(name, init_args, entry_points_caller);
        self.deployed_contracts
            .borrow_mut()
            .push(deployed_contract.clone());
        self.events_count
            .borrow_mut()
            .insert(deployed_contract.clone(), 0);
        deployed_contract
    }

    pub fn call_contract<T: FromBytes + CLTyped>(
        &self,
        address: &Address,
        call_def: CallDef
    ) -> Result<T, OdraError> {
        let backend = self.backend.borrow();

        let use_proxy = T::cl_type() != <()>::cl_type() || !call_def.attached_value().is_zero();
        let call_result = backend.call_contract(address, call_def, use_proxy);

        let mut events_map: BTreeMap<Address, Vec<odra_types::Bytes>> = BTreeMap::new();
        let mut binding = self.events_count.borrow_mut();

        // Go through all contracts and collect their events
        self.deployed_contracts
            .borrow()
            .iter()
            .for_each(|contract_address| {
                let events_count = binding.get_mut(contract_address).unwrap();
                let old_events_last_id = events_count.clone();
                let new_events_count = backend.get_events_count(contract_address);
                let mut events = vec![];
                for event_id in old_events_last_id..new_events_count {
                    let event = backend
                        .get_event(contract_address, event_id as i32)
                        .unwrap();
                    events.push(event);
                }

                events_map.insert(contract_address.clone(), events);

                *events_count = new_events_count;
            });

        self.last_call_result.replace(Some(CallResult {
            contract_address: address.clone(),
            caller: backend.caller(),
            gas_used: backend.last_call_gas_cost(),
            result: call_result.clone(),
            events: events_map
        }));

        call_result.map(|bytes| {
            T::from_bytes(&bytes)
                .map(|(obj, _)| obj)
                .map_err(|_| OdraError::VmError(VmError::Deserialization))
        })?
    }

    pub fn contract_env(&self) -> ContractEnv {
        self.backend.borrow().contract_env()
    }

    pub fn print_gas_report(&self) {
        let backend = self.backend.borrow();
        backend.print_gas_report()
    }

    pub fn balance_of(&self, address: &Address) -> U512 {
        let backend = self.backend.borrow();
        backend.balance_of(address)
    }

    pub fn get_event<T: FromBytes + EventInstance>(
        &self,
        contract_address: &Address,
        index: i32
    ) -> Result<T, EventError> {
        let backend = self.backend.borrow();

        let bytes = backend.get_event(contract_address, index)?;
        T::from_bytes(&bytes)
            .map_err(|_| EventError::Parsing)
            .map(|r| r.0)
    }

    pub fn get_event_bytes(
        &self,
        contract_address: &Address,
        index: i32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.borrow();
        backend.get_event(contract_address, index)
    }

    pub fn contract_event_names(&self, contract_address: &Address) -> Vec<String> {
        let backend = self.backend.borrow();
        let events_count = backend.get_events_count(contract_address);
        let mut event_names = vec![];
        for event_id in 0..events_count {
            let bytes = backend
                .get_event(contract_address, event_id as i32)
                .unwrap();
            let event_name = Self::extract_event_name(&bytes).unwrap();
            event_names.push(event_name);
        }
        event_names
    }

    pub fn events(&self) -> Vec<Bytes> {
        let mut events = vec![];
        self.events_count
            .borrow()
            .iter()
            .for_each(|(contract_address, count)| {
                for i in 0..*count {
                    events.push(self.get_event_bytes(contract_address, i as i32).unwrap());
                }
            });
        events
    }

    pub fn event_names(&self) -> Vec<String> {
        let mut event_names = vec![];
        self.events_count
            .borrow()
            .iter()
            .for_each(|(contract_address, _)| {
                event_names.append(&mut self.contract_event_names(contract_address));
            });
        event_names
    }

    pub fn events_count(&self, contract_address: &Address) -> u32 {
        let backend = self.backend.borrow();
        backend.get_events_count(contract_address)
    }

    pub fn emitted(&self, event_name: &str) -> bool {
        self.event_names().contains(&event_name.to_string())
    }

    pub fn emitted_event<T: ToBytes + EventInstance>(&self, event: &T) -> bool {
        self.events()
            .contains(&Bytes::from(event.to_bytes().unwrap()))
    }

    pub fn last_call(&self) -> CallResult {
        self.last_call_result.borrow().clone().unwrap().clone()
    }
}

impl HostEnv {
    /// Returns the name of the passed event
    pub fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
        let name: String = FromBytes::from_bytes(bytes)
            .map_err(|_| EventError::CouldntExtractName)?
            .0;
        name.strip_prefix("event_")
            .map(|s| s.to_string())
            .ok_or(EventError::UnexpectedType(name))
    }
}
