use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use casper_types::{BlockTime, Motes, PublicKey, SecretKey, U512};
use casper_types::account::AccountHash;
use odra::types::{Address, EventData};
use crate::{CallDef, ModuleCaller};

pub trait HostContext {
    fn set_caller(&mut self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&mut self, time_diff: u64);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&mut self, amount: U512);
    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Vec<u8>;
    fn new_contract(&mut self, contract_id: &str, caller: ModuleCaller) -> Address;
    fn new() -> Self where Self: Sized;
}