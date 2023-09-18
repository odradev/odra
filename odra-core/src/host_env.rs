use odra::prelude::Rc;
use odra::prelude::RefCell;
use odra::types::{Address, Balance, EventData};
use crate::path_stack::PathStack;

pub struct HostEnv {
    path_stack: PathStack,
    backend: Rc<RefCell<dyn HostContext>>,
}

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&self, time_diff: u64);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&self, amount: Balance);
}

impl HostEnv {
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv {
            path_stack: PathStack::new(),
            backend,
        }
    }
    pub fn clone_empty(&self) -> Self {
        HostEnv {
            path_stack: PathStack::new(),
            backend: self.backend.clone(),
        }
    }

    pub fn get_account(&self, index: usize) -> Address {
        let backend = self.backend.borrow();
        backend.get_account(index)
    }

    pub fn set_caller(&mut self, address: Address) {
        let backend = self.backend.borrow_mut();
        backend.set_caller(address)
    }

    pub fn advance_block_time(&mut self, time_diff: u64) {
        let backend = self.backend.borrow_mut();
        backend.advance_block_time(time_diff)
    }

    pub fn attach_value(&mut self, amount: Balance) {
        let backend = self.backend.borrow_mut();
        backend.attach_value(amount)
    }
}