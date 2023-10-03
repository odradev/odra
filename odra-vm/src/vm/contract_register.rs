use odra_core::prelude::{collections::*, *};
use odra_core::HostEnv;
use odra_types::call_def::CallDef;
use odra_types::{casper_types::RuntimeArgs, Address, Bytes, OdraError, VmError};

use super::contract_container::ContractContainer;

#[derive(Default)]
pub struct ContractRegister {
    contracts: BTreeMap<Address, ContractContainer>
}

impl ContractRegister {
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    pub fn call(&self, addr: &Address, call_def: CallDef) -> Result<Bytes, OdraError> {
        // todo: make it better
        self.contracts.get(addr).unwrap().call(call_def)
    }

    // pub fn call_constructor(
    //     &self,
    //     addr: &Address,
    //     entrypoint: String,
    //     args: &RuntimeArgs
    // ) -> Result<Vec<u8>, OdraError> {
    //     self.internal_call(addr, |container| {
    //         container.call_constructor(entrypoint, args)
    //     })
    // }
    //
    // fn internal_call<F: FnOnce(&ContractContainer) -> Result<Vec<u8>, OdraError>>(
    //     &self,
    //     addr: &Address,
    //     call_fn: F
    // ) -> Result<Vec<u8>, OdraError> {
    //     let contract = self.contracts.get(addr);
    //     match contract {
    //         Some(container) => call_fn(container),
    //         None => Err(OdraError::VmError(VmError::InvalidContractAddress))
    //     }
    // }
}
