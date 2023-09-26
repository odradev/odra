use casper_types::U512;
use odra::types::{Address, EventData};
use crate::call_def::CallDef;
use crate::contract_env::ContractEnv;
use crate::module::ModuleCaller;
use crate::odra_result::OdraResult;

pub trait ContractContext {
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>>;
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>);
    fn get_caller(&self) -> Address;

    fn call_contract(
        &mut self,
        address: Address,
        call_def: CallDef,
    ) -> OdraResult<Vec<u8>>;
    fn new_contract(&mut self, contract_id: &str, constructor: ModuleCaller) -> Address;
    fn get_block_time(&self) -> u64;
    fn callee(&self) -> Address;
    fn attached_value(&self) -> Option<U512>;
    fn emit_event(&mut self, event: EventData);
    fn transfer_tokens(&mut self, from: &Address, to: &Address, amount: U512);
    fn balance_of(&self, address: &Address) -> U512;
}

pub trait InitializeBackend {
    fn new() -> Self;
}