use crate::call_def::CallDef;
use crate::{Address, Bytes, OdraError, U512};

pub trait ContractContext {
    fn get_value(&self, key: &[u8]) -> Option<Bytes>;
    fn set_value(&self, key: &[u8], value: Bytes);
    fn caller(&self) -> Address;
    fn self_address(&self) -> Address;
    fn call_contract(&self, address: Address, call_def: CallDef) -> Bytes;
    fn get_block_time(&self) -> u64;
    fn attached_value(&self) -> U512;
    fn emit_event(&self, event: &Bytes);
    fn transfer_tokens(&self, to: &Address, amount: &U512);
    fn revert(&self, error: OdraError) -> !;
}
