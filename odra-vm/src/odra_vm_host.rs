use crate::odra_vm_contract_env::OdraVmContractEnv;
use crate::OdraVm;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::prelude::{collections::*, *};
use odra_core::{CallDef, ContractContext, ContractEnv, HostContext, HostEnv};
use odra_types::casper_types::BlockTime;
use odra_types::{Address, Bytes, EventData, RuntimeArgs, U512};

pub struct OdraVmHost {
    vm: Rc<RefCell<OdraVm>>,
    contract_env: Rc<ContractEnv>
}

impl HostContext for OdraVmHost {
    fn set_caller(&self, caller: Address) {
        self.vm.borrow().set_caller(caller)
    }

    fn get_account(&self, index: usize) -> Address {
        self.vm.borrow().get_account(index)
    }

    fn advance_block_time(&self, time_diff: BlockTime) {
        self.vm.borrow().advance_block_time_by(time_diff.into())
    }

    fn get_event(&self, contract_address: Address, index: i32) -> Option<odra_types::EventData> {
        self.vm.borrow().get_event(contract_address, index)
    }

    fn attach_value(&self, amount: U512) {
        // self.vm.attach_value(amount)
        todo!()
    }

    fn call_contract(&self, address: &Address, call_def: CallDef) -> Bytes {
        self.vm.borrow().call_contract(*address, call_def)
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        // TODO: panic in nice way
        let address = self
            .vm
            .borrow()
            .register_contract(name, entry_points_caller.unwrap());

        if let Some(init_args) = init_args {
            let _: Bytes = self.call_contract(
                &address,
                CallDef::new(String::from("init"), init_args, None)
            );
        }

        address
    }

    fn contract_env(&self) -> ContractEnv {
        self.contract_env.duplicate()
    }
}

impl OdraVmHost {
    pub fn new(vm: Rc<RefCell<OdraVm>>) -> Rc<RefCell<Self>> {
        let contract_env = Rc::new(ContractEnv::new(0, OdraVmContractEnv::new(vm.clone())));
        Rc::new(RefCell::new(Self { vm, contract_env }))
    }
}
