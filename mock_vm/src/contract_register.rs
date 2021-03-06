use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use odra_types::{Address, OdraError, VmError};
use std::collections::HashMap;

use crate::contract_container::ContractContainer;

#[derive(Default)]
pub struct ContractRegister {
    pub contracts: HashMap<Address, ContractContainer>,
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    pub fn call(
        &self,
        addr: &Address,
        entrypoint: String,
        args: RuntimeArgs,
    ) -> Result<Option<Bytes>, OdraError> {
        let contract = self.contracts.get(addr);
        match contract {
            Some(container) => {
                let maybe_error = std::panic::catch_unwind( || {
                    container.call(entrypoint, args)
                });
                if maybe_error.is_err() {
                    Err(OdraError::VmError(VmError::Panic))
                } else {
                    maybe_error.unwrap()
                }
            },
            None => Err(OdraError::VmError(VmError::InvalidContractAddress)),
        }
    }
}
