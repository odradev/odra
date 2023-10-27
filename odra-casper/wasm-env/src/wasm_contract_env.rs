extern crate alloc;

use crate::host_functions;
use casper_types::U512;
use odra_core::{prelude::*, ContractContext, ContractEnv};
use odra_types::{Address, Bytes, EventData};

#[derive(Clone)]
pub struct WasmContractEnv;

impl ContractContext for WasmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        host_functions::get_value(key).map(|v| Bytes::from(v))
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        host_functions::set_value(key, value.as_slice());
    }

    fn caller(&self) -> Address {
        host_functions::caller()
    }

    fn self_address(&self) -> Address {
        host_functions::self_address()
    }

    fn call_contract(&self, address: Address, call_def: odra_core::CallDef) -> Bytes {
        host_functions::call_contract(address, call_def)
    }

    fn get_block_time(&self) -> u64 {
        host_functions::get_block_time()
    }

    fn attached_value(&self) -> U512 {
        host_functions::attached_value()
    }

    fn emit_event(&self, event: &EventData) {
        todo!()
    }

    fn transfer_tokens(&self, to: &Address, amount: &U512) {
        host_functions::transfer_tokens(to, amount);
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
