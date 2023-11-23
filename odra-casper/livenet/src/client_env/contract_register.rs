use odra_core::{casper_types::RuntimeArgs, Address, OdraError, VmError};

use std::collections::BTreeMap;

use super::contract_container::ContractContainer;

#[derive(Default)]
pub struct ContractRegister {
    contracts: BTreeMap<Address, ContractContainer>
}

impl ContractRegister {
    pub fn add(&mut self, container: ContractContainer) {
        self.contracts.insert(container.address(), container);
    }

    pub fn call(
        &self,
        addr: &Address,
        entrypoint: String,
        args: &RuntimeArgs
    ) -> Result<Vec<u8>, OdraError> {
        self.internal_call(addr, |container| container.call(entrypoint, args.clone()))
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
