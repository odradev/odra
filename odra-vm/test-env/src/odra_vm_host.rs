use odra_core::prelude::{collections::*, *};
use odra_core::{CallDef, HostContext, HostEnv};
use odra_types::casper_types::BlockTime;
use odra_types::{Address, Bytes, RuntimeArgs, U512};
use odra_vm_machine::OdraVmMachine;

pub struct OdraVm {
    vm: OdraVmMachine
}

impl HostContext for OdraVm {
    fn set_caller(&mut self, caller: Address) {
        self.vm.set_caller(caller)
    }

    fn get_account(&self, index: usize) -> Address {
        self.vm.get_account(index)
    }

    fn advance_block_time(&mut self, time_diff: BlockTime) {
        self.vm.advance_block_time_by(time_diff.into())
    }

    fn get_event(&self, contract_address: Address, index: i32) -> Option<odra_types::EventData> {
        self.vm.get_event(contract_address, index)
    }

    fn attach_value(&mut self, amount: U512) {
        // self.vm.attach_value(amount)
        todo!()
    }

    fn call_contract(&mut self, address: &Address, call_def: CallDef) -> Bytes {
        self.vm.call_contract(
            *address,
            &*call_def.entry_point,
            &call_def.args,
            call_def.amount
        )
    }

    fn new_contract(
        &mut self,
        contract_id: &str,
        args: &RuntimeArgs,
        constructor: Option<String>
    ) -> Address {
        // self.vm.register_contract()
        todo!()
    }

    fn new_instance() -> Self
    where
        Self: Sized
    {
        Self {
            vm: OdraVmMachine::default()
        }
    }
}
