use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use odra_types::{
    bytesrepr::Bytes,
    Address, CLValue,
};
use odra_types::{EventData, RuntimeArgs, OdraError};

use crate::context::ExecutionContext;
use crate::contract_container::{ContractContainer, EntrypointCall};
use crate::contract_register::ContractRegister;
use crate::storage::Storage;

#[derive(Default)]
pub struct MockVm {
    state: Arc<RwLock<MockVmState>>,
    contract_register: Arc<RwLock<ContractRegister>>,
}

impl MockVm {
    pub fn register_contract(
        &self,
        entrypoints: HashMap<String, EntrypointCall>,
        args: RuntimeArgs,
    ) -> Address {
        // Create a new address.
        let address = { self.state.write().unwrap().next_contract_address() };
        // Check if contract has init.
        let has_init = entrypoints.contains_key("init");

        // Register new contract under the new address.
        {
            let counter = self.state.read().unwrap().contract_counter();
            let contract = ContractContainer::new(&counter.to_string(), entrypoints);
            self.contract_register
                .write()
                .unwrap()
                .add(address.clone(), contract);
        }

        // Call init if needed.
        if has_init {
            self.call_contract(&address, "init", &args, false).unwrap();
        }
        address
    }

    pub fn call_contract(
        &self,
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        _has_return: bool,
    ) -> Option<Bytes> {
        {
            let mut state = self.state.write().unwrap();
            // If only one address on the call_stack, record snapshot.
            state.snapshot();
            // Put the address on stack.
            state.push_address(address);
        }

        // Call contract from register.
        let register = self.contract_register.read().unwrap();
        let result = register.call(address, String::from(entrypoint), args.clone());
        // Drop the address from stack.
        {
            self.state.write().unwrap().pop_address();
        }

        let mut state = self.state.write().unwrap();
        if state.error.is_none() {
            state.drop_snapshot();
            result
        } else {
            state.restore_snapshot();
            None
        }
    }

    pub fn revert(&self, error: OdraError) {
        self.state.write().unwrap().set_error(error)
    }

    pub fn error(&self) -> Option<OdraError> {
        self.state.read().unwrap().error()
    }

    pub fn get_backend_name(&self) -> String {
        self.state.read().unwrap().get_backend_name()
    }

    pub fn caller(&self) -> Address {
        self.state.read().unwrap().caller()
    }

    pub fn set_var(&self, key: &[u8], value: &CLValue) {
        self.state.write().unwrap().set_var(key, value);
    }

    pub fn get_var(&self, key: &[u8]) -> Option<CLValue> {
        self.state.read().unwrap().get_var(key)
    }

    pub fn set_dict_value(&self, dict: &[u8], key: &[u8], value: &CLValue) {
        self.state.write().unwrap().set_dict_value(dict, key, value);
    }

    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Option<CLValue> {
        self.state.read().unwrap().get_dict_value(dict, key)
    }

    pub fn events(&self) -> Vec<EventData> {
        self.state.read().unwrap().events.clone()
    }
}

#[derive(Clone)]
pub struct MockVmState {
    storage: Storage,
    exec_context: ExecutionContext,
    events: Vec<EventData>,
    contract_counter: u32,
    error: Option<OdraError>,
}

impl MockVmState {
    pub fn get_backend_name(&self) -> String {
        "MockVM".to_string()
    }

    pub fn caller(&self) -> Address {
        self.exec_context.previous().clone()
    }

    pub fn set_caller(&mut self, address: &Address) {
        self.pop_address();
        self.push_address(address);
    }

    pub fn set_var(&mut self, key: &[u8], value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage.insert_single_value(&ctx, key, value.clone());
    }

    pub fn get_var(&self, key: &[u8]) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage.get_value(&ctx, key)
    }

    pub fn set_dict_value(&mut self, dict: &[u8], key: &[u8], value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage
            .insert_dict_value(&ctx, dict, key, value.clone());
    }

    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage.get_dict_value(&ctx, dict, key)
    }

    pub fn emit_event(&mut self, event_data: &EventData) {
        self.events.push(event_data.clone());
    }

    pub fn require(expression: bool, msg: &str) {
        if !expression {
            panic!("\x1b[91mRequire failed: {}\x1b[0m", msg);
        }
    }

    pub fn push_address(&mut self, address: &Address) {
        self.exec_context.push(address.clone());
    }

    pub fn pop_address(&mut self) {
        self.exec_context.drop();
    }

    pub fn next_contract_address(&mut self) -> Address {
        let address = Address::new(&self.contract_counter.to_be_bytes());
        self.contract_counter += 1;
        address
    }

    pub fn contract_counter(&self) -> u32 {
        self.contract_counter
    }

    pub fn set_error(&mut self, error: OdraError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    pub fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    fn snapshot(&mut self) {
        if self.exec_context.is_in_caller_context() {
            self.storage.take_snapshot();
        }
    }

    fn drop_snapshot(&mut self) {
        if self.exec_context.is_in_caller_context() {
            self.storage.drop_snapshot();
        }
    }

    fn restore_snapshot(&mut self) {
        if self.exec_context.is_in_caller_context() {
            self.storage.restore_snapshot();
        }
    }
}

impl Default for MockVmState {
    fn default() -> Self {
        let mut backend = MockVmState {
            storage: Default::default(),
            exec_context: Default::default(),
            events: Default::default(),
            contract_counter: 0,
            error: None,
        };
        backend.push_address(default_accounts().first().unwrap());
        backend
    }
}

fn default_accounts() -> Vec<Address> {
    vec![
        Address::new(b"first_address"),
        Address::new(b"second_address"),
        Address::new(b"third_address"),
        Address::new(b"fourth_address"),
        Address::new(b"fifth_address"),
    ]
}

#[cfg(test)]
mod tests {

    // use crate::vm::default_accounts;

    // use super::MockVm;

    // #[test]
    // fn test_default_caller() {
    //     assert_eq!(
    //         MockVm::default().caller(),
    //         default_accounts().first().unwrap().clone()
    //     );
    // }
}
