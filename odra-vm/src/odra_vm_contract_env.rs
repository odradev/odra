use crate::vm::OdraVm;
use blake2::digest::VariableOutput;
use blake2::{Blake2b, Blake2b512, Blake2bVar, Blake2s256, Digest};
use odra_core::casper_types::BlockTime;
use odra_core::casper_types::{
    bytesrepr::{Bytes, ToBytes},
    U512
};
use odra_core::prelude::*;
use odra_core::{casper_types, Address, OdraError};
use odra_core::{CallDef, ContractContext};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Write;

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

    fn self_balance(&self) -> U512 {
        self.vm.borrow().self_balance()
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

    fn get_named_arg_bytes(&self, name: &str) -> Bytes {
        self.vm.borrow().get_named_arg(name).into()
    }

    fn handle_attached_value(&self) {
        // no-op
    }

    fn clear_attached_value(&self) {
        // no-op
    }

    fn hash(&self, bytes: &[u8]) -> [u8; 32] {
        let mut result = [0u8; 32];
        let mut hasher = <Blake2bVar as VariableOutput>::new(32).expect("should create hasher");
        let _ = hasher.write(bytes);
        hasher
            .finalize_variable(&mut result)
            .expect("should copy hash to the result array");
        result
    }
}

impl OdraVmContractEnv {
    pub fn new(vm: Rc<RefCell<OdraVm>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self { vm }))
    }
}
