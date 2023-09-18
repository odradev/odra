use odra_types::{Address, OdraError, VmError, casper_types::RuntimeArgs};

use crate::contract_container::ContractContainer;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ContractRegister {
    contracts: BTreeMap<Address, ContractContainer>
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    pub fn call(
        &self,
        addr: &Address,
        entrypoint: String,
        args: &RuntimeArgs
    ) -> Result<Vec<u8>, OdraError> {
        self.internal_call(addr, |container| container.call(entrypoint, args))
    }

    pub fn call_constructor(
        &self,
        addr: &Address,
        entrypoint: String,
        args: &RuntimeArgs
    ) -> Result<Vec<u8>, OdraError> {
        self.internal_call(addr, |container| {
            container.call_constructor(entrypoint, args)
        })
    }

    fn internal_call<F: FnOnce(&ContractContainer) -> Result<Vec<u8>, OdraError>>(
        &self,
        addr: &Address,
        call_fn: F
    ) -> Result<Vec<u8>, OdraError> {
        let contract = self.contracts.get(addr);
        match contract {
            Some(container) => call_fn(container),
            None => Err(OdraError::VmError(VmError::InvalidContractAddress))
        }
    }
}
