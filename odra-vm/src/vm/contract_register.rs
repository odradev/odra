use odra_core::call_def::CallDef;
use odra_core::prelude::*;
use odra_core::HostEnv;
use odra_core::{casper_types::RuntimeArgs, Address, Bytes, OdraError, VmError};

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
        if let Some(contract) = self.contracts.get(addr) {
            return contract.call(call_def);
        }
        Err(OdraError::VmError(VmError::InvalidContractAddress))
    }
}
