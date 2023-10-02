use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::prelude::{collections::*, *};
use odra_core::{CallDef, ContractContext, ContractEnv, HostContext, HostEnv};
use odra_types::casper_types::BlockTime;
use odra_types::{Address, Bytes, EventData, RuntimeArgs, U512};
use odra_vm_machine::OdraVmMachine;

pub struct OdraVmEnv {
    vm: OdraVmMachine
}

impl ContractContext for OdraVmEnv {
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

    fn emit_event(&mut self, event: EventData) {
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

impl HostContext for OdraVmEnv {
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

    fn call_contract(&mut self, address: &Address, host_env: HostEnv, call_def: CallDef) -> Bytes {
        self.vm.call_contract(*address, host_env, call_def)
    }

    fn new_contract(
        &mut self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        // TODO: panic in nice way
        self.vm
            .register_contract(name, entry_points_caller.unwrap())
        // if init_args.is_some() {
        //     todo!()
        //     // self.vm.call_contract(
        //     //     *address,
        //     //     &*call_def.entry_point,
        //     //     &call_def.args,
        //     //     call_def.amount
        //     // )
        // }
    }

    fn new_instance() -> Self
    where
        Self: Sized
    {
        Self {
            vm: OdraVmMachine::default()
        }
    }

    fn contract_env(&self) -> ContractEnv {
        todo!()
    }
}
