use odra_types::EventData;
use odra_types::{bytesrepr::FromBytes, Address, CLValue};


use super::context::{ExecutionContext, Context};
use super::storage::Storage;

#[derive(Clone)]
pub struct MockVm {
    storage: Storage,
    exec_context: ExecutionContext,
    events: Vec<EventData>,
}

impl MockVm {
    pub fn get_backend_name(&self) -> String {
        "mock_vm".to_string()
    }

    pub fn caller(&self) -> Address {
        self.exec_context.previous().address.clone()
    }

    pub fn set_caller(&mut self, address: &Address) {
        self.pop_address();
        self.push_address(address);
    }

    pub fn set_var(&mut self, key: &[u8], value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage
            .insert_single_value(&ctx.address, key, value.clone());
    }

    pub fn get_var(&self, key: &[u8]) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage.get(&ctx.address, key)
    }

    pub fn set_dict_value(&mut self, dict: &[u8], key: &[u8], value: &CLValue) {
        let ctx = self.exec_context.current();
        self.storage
            .insert_dict_value(&ctx.address, dict, key, value.clone());
    }

    pub fn get_dict_value(&self, dict: &[u8], key: &[u8]) -> Option<CLValue> {
        let ctx = self.exec_context.current();
        self.storage.get_dict_value(&ctx.address, dict, key)
    }

    pub fn emit_event(&mut self, event_data: &EventData) {
        self.events.push(event_data.clone());
    }

    pub fn event(&self, at: i32) -> EventData {
        self.events.get(self.index_to_usize(at)).unwrap().clone()
    }

    pub fn assert_event_type_emitted(&self, event_name: &str) {
        for event in self.events.clone().into_iter() {
            if MockVm::event_name(event.as_slice()) == *event_name {
                return;
            }
        }

        // TODO: better message
        assert_eq!(event_name, "");
    }

    pub fn assert_event_emitted(&self, event_data: &EventData) {
        for event in self.events.clone().into_iter() {
            if event == *event_data {
                return;
            }
        }

        // TODO: better message
        panic!("Event not found")
    }

    pub fn assert_event(&self, event_data: &EventData, at: i32) {
        assert_eq!(self.event(at), event_data.clone())
    }

    pub fn assert_event_type(&self, event_name: &str, at: i32) {
        assert_eq!(
            MockVm::event_name(self.event(at).as_slice()),
            event_name
        );
    }

    pub fn assert_event_type_not_emitted(&self, event_name: &str) {
        for event in self.events.clone().into_iter() {
            if MockVm::event_name(event.as_slice()) == *event_name {
                // TODO: better message
                assert_eq!(event_name, "");
            }
        }
    }

    pub fn assert_event_not_emitted(&self, event_data: &EventData) {
        for event in self.events.clone().into_iter() {
            if event == *event_data {
                // TODO: better message
                panic!("Event not found")
            }
        }
    }

    pub fn require(expression: bool, msg: &str) {
        if !expression {
            panic!("\x1b[91mRequire failed: {}\x1b[0m", msg);
        }
    }
}

impl Default for MockVm {
    fn default() -> Self {
        let mut backend = MockVm {
            storage: Default::default(),
            exec_context: Default::default(),
            events: Default::default(),
        };
        backend.push_address(default_accounts().first().unwrap());
        backend
    }
}

impl MockVm {
    fn index_to_usize(&self, index: i32) -> usize {
        if index.is_negative() {
            self.events.len() - index.wrapping_abs() as usize
        } else {
            index as usize
        }
    }

    fn event_name(event_data: &[u8]) -> String {
        let (name, _): (String, _) = FromBytes::from_bytes(event_data).unwrap();
        name
    }

    pub fn push_address(&mut self, address: &Address) {
        self.exec_context.push(Context::from(address.clone()));
    }

    pub fn pop_address(&mut self) {
        self.exec_context.drop();
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

    use crate::vm::default_accounts;

    use super::MockVm;

    #[test]
    fn test_default_caller() {
        assert_eq!(
            MockVm::default().caller(),
            default_accounts().first().unwrap().clone()
        );
    }
}
