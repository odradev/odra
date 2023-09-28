use crate::host_context::HostContext;
use crate::prelude::{Rc, RefCell};
use crate::ModuleCaller;
use casper_types::U512;
use odra_types::Address;

pub struct HostEnv {
    backend: Rc<RefCell<dyn HostContext>>
}

impl HostEnv {
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv {
            backend,
        }
    }

    pub fn clone_empty(&self) -> Self {
        HostEnv {
            backend: self.backend.clone()
        }
    }

    pub fn get_account(&self, index: usize) -> Address {
        let backend = self.backend.borrow();
        backend.get_account(index)
    }

    pub fn set_caller(&mut self, address: Address) {
        let mut backend = self.backend.borrow_mut();
        backend.set_caller(address)
    }

    pub fn advance_block_time(&mut self, time_diff: u64) {
        let mut backend = self.backend.borrow_mut();
        backend.advance_block_time(time_diff)
    }

    pub fn attach_value(&mut self, amount: U512) {
        let mut backend = self.backend.borrow_mut();
        backend.attach_value(amount)
    }

    pub fn new_contract(&mut self, contract_id: &str, caller: ModuleCaller) -> Address {
        let mut backend = self.backend.borrow_mut();
        backend.new_contract(contract_id, caller)
    }
}
