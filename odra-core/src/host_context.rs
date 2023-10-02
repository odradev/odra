use crate::prelude::*;
use crate::{CallDef, HostEnv};
use odra_types::casper_types::BlockTime;
use odra_types::Bytes;
use odra_types::RuntimeArgs;
use odra_types::{Address, EventData, U512};

pub trait HostContext {
    fn set_caller(&mut self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&mut self, time_diff: BlockTime);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&mut self, amount: U512);
    fn call_contract(&mut self, address: &Address, call_def: CallDef) -> Bytes;
    fn new_contract(
        &mut self,
        contract_id: &str,
        args: &RuntimeArgs,
        constructor: Option<String>
    ) -> Address;
    fn new_instance() -> Self
    where
        Self: Sized;
    fn new() -> HostEnv
    where
        Self: Sized + 'static
    {
        HostEnv::new(Rc::new(RefCell::new(Self::new_instance())))
    }
}
