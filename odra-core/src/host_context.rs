use crate::entry_point_callback::EntryPointsCaller;
use crate::prelude::*;
use crate::{CallDef, ContractEnv, HostEnv};
use odra_types::casper_types::BlockTime;
use odra_types::Bytes;
use odra_types::RuntimeArgs;
use odra_types::{Address, EventData, U512};

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn advance_block_time(&self, time_diff: BlockTime);
    fn get_event(&self, contract_address: Address, index: i32) -> Option<EventData>;
    fn attach_value(&self, amount: U512);
    fn call_contract(&self, address: &Address, host_env: HostEnv, call_def: CallDef) -> Bytes;
    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address;
    fn contract_env(&self) -> ContractEnv;
}
