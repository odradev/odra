use casper_types::U512;
use odra::prelude::Rc;
use odra::prelude::RefCell;
use odra::types::{Address, EventData};
use crate::path_stack::PathStack;

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&self, time_diff: u64);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&self, amount: U512);
}