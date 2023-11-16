use crate::entry_point_callback::EntryPointsCaller;
use crate::event::EventError;
use crate::host_context::HostContext;
use crate::prelude::*;
use crate::{CallDef, ContractEnv};
use casper_event_standard::EventInstance;
use odra_types::{Address, U512, OdraError, VmError};
use odra_types::RuntimeArgs;
use odra_types::{CLTyped, FromBytes};

#[derive(Clone)]
pub struct HostEnv {
    backend: Rc<RefCell<dyn HostContext>>
}

impl HostEnv {
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv { backend }
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
        backend.new_contract(name, init_args, entry_points_caller)
    }

    pub fn call_contract<T: FromBytes + CLTyped>(&self, address: &Address, call_def: CallDef) -> Result<T, OdraError> {
        let backend = self.backend.borrow();
        let use_proxy = T::cl_type() != <()>::cl_type() || !call_def.attached_value().is_zero();
        let call_result = backend.call_contract(address, call_def, use_proxy);
        call_result.map(|bytes| T::from_bytes(&bytes)
            .map(|(obj, _)| obj)
            .map_err(|_| OdraError::VmError(VmError::Deserialization))
        )?
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
        let event_name = Self::extract_event_name(&bytes)?;
        if event_name == format!("event_{}", T::name()) {
            T::from_bytes(&bytes)
                .map_err(|_| EventError::Parsing)
                .map(|r| r.0)
        } else {
            Err(EventError::UnexpectedType(event_name))
        }
    }

    /// Returns the name of the passed event
    fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
        let name = FromBytes::from_bytes(bytes).map_err(|_| EventError::CouldntExtractName)?;
        Ok(name.0)
    }
}
