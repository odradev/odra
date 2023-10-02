use crate::entry_point_callback::EntryPointsCaller;
use crate::host_context::HostContext;
use crate::prelude::*;
use crate::{CallDef, ContractEnv};
use odra_types::casper_types::BlockTime;
use odra_types::Address;
use odra_types::FromBytes;
use odra_types::{RuntimeArgs, U512};

#[derive(Clone)]
pub struct HostEnv {
    backend: Rc<RefCell<dyn HostContext>>
}

impl HostEnv {
    pub fn new(backend: Rc<RefCell<dyn HostContext>>) -> HostEnv {
        HostEnv { backend }
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

    pub fn set_caller(&self, address: Address) {
        let mut backend = self.backend.borrow_mut();
        backend.set_caller(address)
    }

    pub fn advance_block_time(&self, time_diff: BlockTime) {
        let mut backend = self.backend.borrow_mut();
        backend.advance_block_time(time_diff)
    }

    pub fn attach_value(&self, amount: U512) {
        let mut backend = self.backend.borrow_mut();
        backend.attach_value(amount)
    }

    pub fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        let mut backend = self.backend.borrow_mut();
        backend.new_contract(name, init_args, entry_points_caller)
    }

    pub fn call_contract<T: FromBytes>(&self, address: &Address, call_def: CallDef) -> T {
        let mut backend = self.backend.borrow_mut();
        let result = backend.call_contract(address, self.clone_empty(), call_def);
        T::from_bytes(&result).unwrap().0
    }

    pub fn contract_env(&self) -> ContractEnv {
        todo!()
        // let mut backend = self.backend.borrow_mut();
        // ContractEnv::new(0, )
        // backend.contract_env()
    }
}
