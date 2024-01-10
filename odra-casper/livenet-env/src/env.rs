use odra_core::event::EventError;
use odra_core::prelude::*;
use odra_core::{
    Address, Bytes, CallDef, ContractEnv, EntryPointsCaller, HostContext, OdraError, PublicKey,
    RuntimeArgs, U512
};

pub struct LivenetEnv {}

impl LivenetEnv {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new_instance()))
    }

    pub fn new_instance() -> Self {
        Self {}
    }
}

impl HostContext for LivenetEnv {
    fn set_caller(&self, caller: Address) {
        todo!()
    }

    fn set_gas(&self, gas: u64) {
        todo!()
    }

    fn caller(&self) -> Address {
        todo!()
    }

    fn get_account(&self, index: usize) -> Address {
        todo!()
    }

    fn balance_of(&self, address: &Address) -> U512 {
        todo!()
    }

    fn advance_block_time(&self, time_diff: u64) {
        panic!("Cannot advance block time in LivenetEnv")
    }

    fn get_event(&self, contract_address: &Address, index: i32) -> Result<Bytes, EventError> {
        todo!()
    }

    fn get_events_count(&self, contract_address: &Address) -> u32 {
        todo!()
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        _use_proxy: bool
    ) -> Result<Bytes, OdraError> {
        todo!()
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address {
        todo!()
    }

    fn contract_env(&self) -> ContractEnv {
        panic!("Cannot get contract env in LivenetEnv")
    }

    fn print_gas_report(&self) {
        todo!();
        println!("Gas report:");
    }

    fn last_call_gas_cost(&self) -> u64 {
        todo!()
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        todo!()
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        todo!()
    }
}
