use crate::entry_point_callback::EntryPointsCaller;
use crate::{CallDef, ContractEnv};
use odra_types::Bytes;
use odra_types::RuntimeArgs;
use odra_types::{Address, EventData, U512};

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn balance_of(&self, address: &Address) -> U512;
    fn advance_block_time(&self, time_diff: u64);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn call_contract(&self, address: &Address, call_def: CallDef) -> Bytes;
    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address;
    fn contract_env(&self) -> ContractEnv;
    fn print_gas_report(&self);
}
