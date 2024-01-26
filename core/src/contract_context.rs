use crate::call_def::CallDef;
use crate::{Address, Bytes, OdraError, U512};

#[cfg_attr(test, allow(unreachable_code))]
#[cfg_attr(test, mockall::automock)]
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
    fn get_named_arg_bytes(&self, name: &str) -> Bytes;
    fn handle_attached_value(&self);
    fn clear_attached_value(&self);
    fn hash(&self, bytes: &[u8]) -> [u8; 32];
}
