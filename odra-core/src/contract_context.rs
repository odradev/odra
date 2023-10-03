use crate::prelude::*;
use odra_types::call_def::CallDef;
use odra_types::casper_types::BlockTime;
use odra_types::{Address, Bytes, EventData, U512};

pub trait ContractContext {
    fn get_value(&self, key: &[u8]) -> Option<Bytes>;
    fn set_value(&self, key: &[u8], value: Bytes);
    fn caller(&self) -> Address;
    fn call_contract(&mut self, address: Address, call_def: CallDef) -> Bytes;
    fn get_block_time(&self) -> BlockTime;
    fn callee(&self) -> Address;
    fn attached_value(&self) -> Option<U512>;
    fn emit_event(&mut self, event: EventData);
    fn transfer_tokens(&mut self, from: &Address, to: &Address, amount: U512);
    fn balance_of(&self, address: &Address) -> U512;
    fn revert(&self, code: u16) -> !;
    // fn install_contract(
    //     entry_points: EntryPoints,
    // ) -> Address;
    // fn named_arg(&self, name: &str) -> Option<Vec<u8>>;
}
