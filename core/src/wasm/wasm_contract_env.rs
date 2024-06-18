//! WasmContractEnv is an implementation of ContractContext for Wasm environment.

use crate::casper_types;
use crate::casper_types::bytesrepr::Bytes;
use crate::casper_types::{CLValue, BLAKE2B_DIGEST_LENGTH};
use crate::wasm::host_functions;
use crate::{Address, OdraError};
use crate::{ContractContext, ContractEnv, ExecutionError, OdraResult};
use casper_types::U512;

/// ContractContext implementation for Wasm environment.
#[derive(Clone)]
pub struct WasmContractEnv;

impl ContractContext for WasmContractEnv {
    fn get_value(&self, key: &[u8]) -> Option<Bytes> {
        host_functions::get_value(key).map(Bytes::from)
    }

    fn set_value(&self, key: &[u8], value: Bytes) {
        host_functions::set_value(key, value.as_slice());
    }

    fn get_named_value(&self, name: &str) -> Option<Bytes> {
        host_functions::get_named_key(name)
    }

    fn set_named_value(&self, name: &str, value: CLValue) {
        host_functions::set_named_key(name, value);
    }

    fn get_dictionary_value(&self, dictionary_name: &str, key: &[u8]) -> Option<Bytes> {
        host_functions::get_dictionary_value(dictionary_name, key)
    }

    fn set_dictionary_value(&self, dictionary_name: &str, key: &[u8], value: CLValue) {
        host_functions::set_dictionary_value(dictionary_name, key, value);
    }

    fn remove_dictionary(&self, dictionary_name: &str) {
        host_functions::remove_dictionary(dictionary_name);
    }

    fn caller(&self) -> Address {
        host_functions::caller()
    }

    fn self_address(&self) -> Address {
        host_functions::self_address()
    }

    fn call_contract(&self, address: Address, call_def: crate::CallDef) -> Bytes {
        host_functions::call_contract(address, call_def)
    }

    fn get_block_time(&self) -> u64 {
        host_functions::get_block_time()
    }

    fn attached_value(&self) -> U512 {
        host_functions::attached_value()
    }

    fn self_balance(&self) -> U512 {
        host_functions::self_balance()
    }

    fn emit_event(&self, event: &Bytes) {
        host_functions::emit_event(event);
    }

    fn transfer_tokens(&self, to: &Address, amount: &U512) {
        host_functions::transfer_tokens(to, amount);
    }

    fn revert(&self, error: OdraError) -> ! {
        host_functions::revert(error.code())
    }

    fn get_named_arg_bytes(&self, name: &str) -> OdraResult<Bytes> {
        host_functions::get_named_arg(name)
            .map(Bytes::from)
            .map_err(|_| OdraError::ExecutionError(ExecutionError::MissingArg))
    }

    fn get_opt_named_arg_bytes(&self, name: &str) -> Option<Bytes> {
        host_functions::get_named_arg(name).ok().map(Bytes::from)
    }

    fn handle_attached_value(&self) {
        host_functions::handle_attached_value();
    }

    fn clear_attached_value(&self) {
        host_functions::clear_attached_value();
    }

    fn hash(&self, bytes: &[u8]) -> [u8; BLAKE2B_DIGEST_LENGTH] {
        host_functions::blake2b(bytes)
    }
}

impl WasmContractEnv {
    /// Creates new ContractEnv with WasmContractEnv as backend.
    pub fn new_env() -> ContractEnv {
        ContractEnv::new(0, WasmContractEnv)
    }

    /// Borrows self.
    pub fn borrow(&self) -> &Self {
        self
    }
}
