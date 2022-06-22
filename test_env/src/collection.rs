use odra_types::Address;
#[cfg(feature = "mock-vm")]
use odra_types::{bytesrepr::Bytes, RuntimeArgs};
use std::collections::HashMap;

use super::container::ContractContainer;

#[derive(Default)]
pub struct ContractCollection {
    pub contracts: HashMap<Address, ContractContainer>,
}

impl ContractCollection {
    pub fn new() -> Self {
        ContractCollection {
            contracts: HashMap::new(),
        }
    }

    pub fn add(&mut self, addr: Address, container: ContractContainer) {
        self.contracts.insert(addr, container);
    }

    #[cfg(feature = "mock-vm")]
    pub fn call(&self, addr: &Address, entrypoint: String, args: RuntimeArgs) -> Option<Bytes> {
        self.contracts.get(addr).unwrap().call(entrypoint, args)
    }
}
