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
        constructor: Option<(String, RuntimeArgs, EntrypointCall)>,
        entrypoints: HashMap<String, EntrypointCall>,
    ) -> Address {
        // Create a new address.
        let address = { self.state.write().unwrap().next_contract_address() };
        // Check if contract has init.
        let has_init = constructor.is_some();

        let original_entrypoints = entrypoints.to_owned();

        let contract_namespace = self.state.read().unwrap().get_contract_namespace();

        let constructor_entrypoint = constructor
            .clone()
            .and_then(|(constructor_name, _, call)| Some([(constructor_name, call)]));

        let entrypoints = match constructor_entrypoint {
            Some(constructor) => constructor.into_iter().chain(entrypoints).collect::<HashMap<_, _>>(),
            None => entrypoints,
        };

        // Register new contract under the new address.
        {
            let contract = ContractContainer::new(&contract_namespace, entrypoints);
            self.contract_register
                .write()
                .unwrap()
                .add(address.clone(), contract);
        }

        // Call init if needed.
        if has_init {
            let (constructor_name, args, _) = constructor.unwrap();

            self.call_contract(&address, &constructor_name, &args, false);
            let contract = ContractContainer::new(&contract_namespace, original_entrypoints);
            self.contract_register
                .write()
                .unwrap()
                .add(address.clone(), contract);
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
            if state.is_in_caller_context() {
                state.take_snapshot();
                state.clear_error();
            }
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
            // If only one address on the call_stack, drop the snapshot
            if state.is_in_caller_context() {
                state.drop_snapshot();
            }
            result
        } else {
            // If only one address on the call_stack an an error occurred, restore the snapshot
            if state.is_in_caller_context() {
                state.restore_snapshot();
            }
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

    pub fn get_contract_namespace(&self) -> String {
        self.contract_counter.to_string()
    }

    pub fn set_error(&mut self, error: OdraError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    fn is_in_caller_context(&self) -> bool {
        self.exec_context.len() == 1
    }

    fn take_snapshot(&mut self) {
        self.storage.take_snapshot();
    }

    fn drop_snapshot(&mut self) {
        self.storage.drop_snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.storage.restore_snapshot();
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
