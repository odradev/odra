use odra_core::{casper_types::RuntimeArgs, Address, OdraResult, VmError};

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
    ) -> OdraResult<Vec<u8>> {
        self.internal_call(addr, |container| container.call(entrypoint, args.clone()))
    }

    fn internal_call<F: FnOnce(&ContractContainer) -> OdraResult<Vec<u8>>>(
        &self,
        addr: &Address,
        call_fn: F
    ) -> OdraResult<Vec<u8>> {
        let contract = self.contracts.get(addr);
        match contract {
            Some(container) => call_fn(container),
            None => Err(OdraError::VmError(VmError::InvalidContractAddress))
        }
    }
}
