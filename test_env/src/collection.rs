use std::collections::HashMap;

use odra_types::{bytesrepr::Bytes, Address, RuntimeArgs};

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

    pub fn call(&self, addr: &Address, entrypoint: String, args: RuntimeArgs) -> Bytes {
        self.contracts.get(addr).unwrap().call(entrypoint, args)
    }
}
