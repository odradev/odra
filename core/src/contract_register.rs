use crate::call_def::CallDef;
use crate::ContractContainer;
use crate::{casper_types::bytesrepr::Bytes, Address, OdraError, VmError};
use crate::{prelude::*, OdraResult};

/// A struct representing a contract register that maps an address to a contract.
///
/// A register is a central place where all contracts are stored. It is used by the
/// host side to manage and/or call contracts.
#[derive(Default)]
pub struct ContractRegister {
    contracts: BTreeMap<Address, ContractContainer>
}

impl ContractRegister {
    /// Adds a contract to the register.
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    /// Calls the entry point with the given call definition.
    ///
    /// Returns bytes representing the result of the call or an error if the address
    /// is not present in the register.
    pub fn call(&self, addr: &Address, call_def: CallDef) -> OdraResult<Bytes> {
        if let Some(contract) = self.contracts.get(addr) {
            return contract.call(call_def);
        }
        Err(OdraError::VmError(VmError::InvalidContractAddress))
    }

    pub fn get(&self, addr: &Address) -> Option<&str> {
        match self.contracts.get(addr) {
            Some(contract) => Some(contract.name()),
            None => None
        }
    }
}
