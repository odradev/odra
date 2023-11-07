use crate::entry_point_callback::EntryPointsCaller;
use crate::event::EventError;
use crate::{CallDef, ContractEnv};
use odra_types::Bytes;
use odra_types::RuntimeArgs;
use odra_types::{Address, U512};

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn get_account(&self, index: usize) -> Address;
    fn balance_of(&self, address: &Address) -> U512;
    fn advance_block_time(&self, time_diff: u64);
    /// Returns event bytes by contract address and index.
    fn get_event(&self, contract_address: &Address, index: i32) -> Result<Bytes, EventError>;
    fn call_contract(&self, address: &Address, call_def: CallDef, use_proxy: bool) -> Bytes;
    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address;
    fn contract_env(&self) -> ContractEnv;
    fn print_gas_report(&self);
}
