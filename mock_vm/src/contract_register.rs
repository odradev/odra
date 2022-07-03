use odra_types::Address;
use odra_types::{bytesrepr::Bytes, RuntimeArgs};
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

    pub fn call(&self, addr: &Address, entrypoint: String, args: RuntimeArgs) -> Option<Bytes> {
        self.contracts.get(addr).unwrap().call(entrypoint, args)
    }
}
