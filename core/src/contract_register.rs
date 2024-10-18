use crate::call_def::CallDef;
use crate::prelude::*;
use crate::ContractContainer;
use crate::{casper_types::bytesrepr::Bytes, VmError};

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

    /// Post install hook.
    pub fn post_install(&mut self, addr: &Address) {
        if let Some(contract) = self.contracts.get_mut(addr) {
            contract.post_install();
        }
    }
}
