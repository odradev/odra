use odra::types::{Address, Balance, EventData};
use crate::call_def::CallDef;
use crate::contract_env::ContractEnv;
use crate::module::ModuleCaller;
use crate::odra_result::OdraResult;

pub trait ContractContext {
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>>;
    fn set(&self, key: Vec<u8>, value: Vec<u8>);
    fn get_caller(&self) -> Address;

    fn call_contract(
        &self,
        env: ContractEnv,
        address: Address,
        call_def: CallDef,
    ) -> OdraResult<Vec<u8>>;
    fn new_contract(&self, constructor: ModuleCaller) -> Address;
    fn get_block_time(&self) -> u64;
    fn callee(&self) -> Address;
    fn attached_value(&self) -> Option<Balance>;
    fn emit_event(&self, event: EventData);
    fn transfer_tokens(&self, from: &Address, to: &Address, amount: Balance);
    fn balance_of(&self, address: &Address) -> Balance;
}
