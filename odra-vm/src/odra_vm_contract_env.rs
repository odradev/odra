use crate::vm::OdraVm;
use odra_core::casper_types::BlockTime;
use odra_core::prelude::*;
use odra_core::{casper_types, Address, Bytes, OdraError, ToBytes, U512};
use odra_core::{CallDef, ContractContext};

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

    fn self_address(&self) -> Address {
        self.vm.borrow().self_address()
    }

    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes {
        self.vm.borrow().call_contract(address, call_def)
    }

    fn get_block_time(&self) -> u64 {
        self.vm.borrow().get_block_time()
    }

    fn attached_value(&self) -> U512 {
        self.vm.borrow().attached_value()
    }

    fn emit_event(&self, event: &Bytes) {
        self.vm.borrow().emit_event(event);
    }

    fn transfer_tokens(&self, to: &Address, amount: &U512) {
        self.vm.borrow().transfer_tokens(to, amount)
    }

    fn revert(&self, error: OdraError) -> ! {
        self.vm.borrow().revert(error)
    }
}

impl OdraVmContractEnv {
    pub fn new(vm: Rc<RefCell<OdraVm>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
