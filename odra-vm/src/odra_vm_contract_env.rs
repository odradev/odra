use crate::vm::OdraVm;
use odra_core::prelude::*;
use odra_core::{CallDef, ContractContext};
use odra_types::casper_types::BlockTime;
use odra_types::{casper_types, Address, Bytes, U512};

pub struct OdraVmContractEnv {
    vm: Rc<RefCell<OdraVm>>
}

impl ContractContext for OdraVmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        self.vm.borrow().get_var(key)
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        self.vm.borrow().set_var(key, value)
    }

    fn caller(&self) -> Address {
        self.vm.borrow().caller()
    }

    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Bytes {
        todo!()
    }

    fn get_block_time(&self) -> BlockTime {
        todo!()
    }

    fn callee(&self) -> Address {
        todo!()
    }

    fn attached_value(&self) -> Option<U512> {
        todo!()
    }

    fn emit_event(&mut self, event: odra_types::EventData) {
        todo!()
    }

    fn transfer_tokens(&mut self, from: &Address, to: &Address, amount: U512) {
        todo!()
    }

    fn balance_of(&self, address: &Address) -> U512 {
        todo!()
    }

    fn revert(&self, code: u16) -> ! {
        todo!()
    }
}

impl OdraVmContractEnv {
    pub fn new(vm: Rc<RefCell<OdraVm>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
