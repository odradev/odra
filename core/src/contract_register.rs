use crate::call_def::CallDef;
use crate::prelude::*;
use crate::ContractContainer;
use crate::{casper_types::bytesrepr::Bytes, VmError};

pub type ContractVersion = u32;

/// A struct representing a contract register that maps an address to a contract.
///
/// A register is a central place where all contracts are stored. It is used by the
/// host side to manage and/or call contracts.
#[derive(Default)]
pub struct ContractRegister {
    contracts: BTreeMap<(Address, ContractVersion), ContractContainer>,
    named_contracts: BTreeMap<String, Address>,
    versions_count: BTreeMap<Address, ContractVersion>
}

impl ContractRegister {
    /// Adds a contract to the register.
    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        let new_version = match self.versions_count.get(&addr) {
            Some(version_number) => version_number + 1,
            None => 0
        };
        self.contracts.insert((addr, new_version), container);
    }

    /// Calls the entry point with the given call definition.
    ///
    /// Returns bytes representing the result of the call or an error if the address
    /// is not present in the register.
    pub fn call(&self, addr: &Address, call_def: CallDef) -> OdraResult<Bytes> {
        if let Some(contract) = self.get(addr) {
            return contract.call(call_def);
        }
        Err(OdraError::VmError(VmError::InvalidContractAddress))
    }

    /// Returns the latest contract container for the given address.
    pub fn get(&self, addr: &Address) -> Option<&ContractContainer> {
        let latest_version = self.versions_count.get(addr).unwrap_or(&0);
        self.contracts.get(&(*addr, *latest_version))
    }

    /// Returns the name of the contract at the given address.
    pub fn get_name(&self, addr: &Address) -> Option<&str> {
        let latest_version = self.versions_count.get(addr).unwrap_or(&0);
        match self.contracts.get(&(*addr, *latest_version)) {
            Some(contract) => Some(contract.name()),
            None => None
        }
    }
    
    /// Returns the address of the contract with the given name.
    pub fn get_address(&self, name: &str) -> Option<Address> {
        self.contracts.iter().find(|contract| {
            contract.1.name() == name
        }).and_then(|a| Some(a.0.0.clone()))
    }

    /// Returns the latest contract container for the given address.
    pub fn get_mut(&mut self, addr: &Address) -> Option<&mut ContractContainer> {
        let latest_version = self.versions_count.get(addr).unwrap_or(&0);
        self.contracts.get_mut(&(*addr, *latest_version))
    }

    /// Post install hook.
    pub fn post_install(&mut self, addr: &Address) {
        if let Some(contract) = self.get_mut(addr) {
            contract.post_install();
        }
    }
}
