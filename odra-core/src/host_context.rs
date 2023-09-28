use crate::prelude::*;
use crate::{CallDef, ModuleCaller};
use odra_types::{Address, EventData, U512};

pub trait HostContext {
    fn set_caller(&mut self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&mut self, time_diff: u64);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&mut self, amount: U512);
    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Vec<u8>;
    fn new_contract(&mut self, contract_id: &str, caller: ModuleCaller) -> Address;
    fn new_instance() -> Self
    where
        Self: Sized;
}
