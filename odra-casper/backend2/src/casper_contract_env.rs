extern crate alloc;

use crate::wasm_host;
use odra_core::{prelude::*, ContractContext, ContractEnv};
use odra_types::Bytes;

#[derive(Clone)]
pub struct WasmContractEnv;

impl ContractContext for WasmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        wasm_host::get_value(key).map(|v| Bytes::from(v))
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        wasm_host::set_value(key, value.as_slice());
    }

    fn caller(&self) -> odra_types::Address {
        wasm_host::caller()
    }

    fn call_contract(
        &mut self,
        address: odra_types::Address,
        call_def: odra_core::CallDef
    ) -> Bytes {
        todo!()
    }

    fn get_block_time(&self) -> casper_types::BlockTime {
        todo!()
    }

    fn callee(&self) -> odra_types::Address {
        todo!()
    }

    fn attached_value(&self) -> Option<casper_types::U512> {
        todo!()
    }

    fn emit_event(&mut self, event: odra_types::EventData) {
        todo!()
    }

    fn transfer_tokens(
        &mut self,
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
