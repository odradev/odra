use odra_core::prelude::*;
use odra_core::{CallDef, ContractContext};
use odra_types::casper_types::BlockTime;
use odra_types::{casper_types, Address, U512};
use odra_vm_machine::OdraVmMachine;

pub struct OdraVmContractEnv {
    vm: Rc<RefCell<OdraVmMachine>>
}

impl ContractContext for OdraVmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    fn set_value(&self, key: &[u8], value: &[u8]) {
        todo!()
    }

    fn caller(&self) -> Address {
        todo!()
    }

    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Vec<u8> {
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
