use crate::entry_point_callback::EntryPointsCaller;
use crate::event::EventError;
use crate::RuntimeArgs;
use crate::{Address, U512};
use crate::{Bytes, OdraError};
use crate::{CallDef, ContractEnv};

pub trait HostContext {
    fn set_caller(&self, caller: Address);
    fn caller(&self) -> Address;
    fn get_account(&self, index: usize) -> Address;
    fn balance_of(&self, address: &Address) -> U512;
    fn advance_block_time(&self, time_diff: u64);
    /// Returns event bytes by contract address and index.
    fn get_event(&self, contract_address: &Address, index: i32) -> Result<Bytes, EventError>;
    fn get_events_count(&self, contract_address: &Address) -> u32;
    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> Result<Bytes, OdraError>;
    fn new_contract(
        &self,
        name: &str,
        init_args: Option<RuntimeArgs>,
        entry_points_caller: Option<EntryPointsCaller>
    ) -> Address;
    fn contract_env(&self) -> ContractEnv;
    fn print_gas_report(&self);
    fn last_call_gas_cost(&self) -> u64;
}
