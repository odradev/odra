extern crate alloc;

use crate::host_functions;
use odra_core::{prelude::*, ContractContext, ContractEnv};
use odra_types::Bytes;

#[derive(Clone)]
pub struct WasmContractEnv;

impl ContractContext for WasmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        host_functions::get_value(key).map(|v| Bytes::from(v))
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        host_functions::set_value(key, value.as_slice());
    }

    fn caller(&self) -> odra_types::Address {
        host_functions::caller()
    }

    fn call_contract(&self, address: odra_types::Address, call_def: odra_core::CallDef) -> Bytes {
        host_functions::call_contract(address, call_def)
    }

    fn get_block_time(&self) -> u64 {
        todo!()
    }

    fn callee(&self) -> odra_types::Address {
        todo!()
    }

    fn attached_value(&self) -> casper_types::U512 {
        host_functions::attached_value()
    }

    fn emit_event(&self, event: odra_types::EventData) {
        todo!()
    }

    fn transfer_tokens(
        &self,
        from: &odra_types::Address,
        to: &odra_types::Address,
        amount: casper_types::U512
    ) {
        todo!()
    }

    fn balance_of(&self, address: &odra_types::Address) -> casper_types::U512 {
        todo!()
    }

    fn revert(&self, code: u16) -> ! {
        todo!()
    }
}

impl WasmContractEnv {
    pub fn new() -> ContractEnv {
        ContractEnv::new(0, Rc::new(RefCell::new(WasmContractEnv)))
    }
}
