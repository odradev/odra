use odra_mock_vm_types::{
    odra_types::{OdraError, VmError},
    Address, Bytes, CallArgs,
};

use crate::contract_container::ContractContainer;
use std::collections::HashMap;

#[derive(Default)]
pub struct ContractRegister {
    contracts: HashMap<Address, ContractContainer>,
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    pub fn call(
        &self,
        addr: &Address,
        entrypoint: String,
        args: CallArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        self.internal_call(addr, |container| {
            std::panic::catch_unwind(|| container.call(entrypoint, args))?
        })
    }

    pub fn call_constructor(
        &self,
        addr: &Address,
        entrypoint: String,
        args: CallArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        self.internal_call(addr, |container| {
            std::panic::catch_unwind(|| container.call_constructor(entrypoint, args))?
        })
    }

    fn internal_call<F: FnOnce(&ContractContainer) -> Result<Option<Bytes>, OdraError>>(
        &self,
        addr: &Address,
        call_fn: F,
    ) -> Result<Option<Bytes>, OdraError> {
        let contract = self.contracts.get(addr);
        match contract {
            Some(container) => call_fn(container),
            None => Err(OdraError::VmError(VmError::InvalidContractAddress)),
        }
    }
}
