//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
use std::collections::HashMap;

use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_casper_types::{Address, Balance, CallArgs, OdraType};
use odra_types::{
    event::{EventError, OdraEvent},
    OdraError
};

use crate::{client_env, EntrypointArgs, EntrypointCall};

/// Returns backend name.
pub fn backend_name() -> String {
    "CasperLivenet".to_string()
}

/// Deploy WASM file with arguments.
pub fn register_contract(
    address: Address,
    entrypoints: HashMap<String, (EntrypointArgs, EntrypointCall)>
) {
    client_env::register_contract(address, entrypoints);
}

/// Call contract under a given address.
pub fn call_contract<T: OdraType>(
    _addr: Address,
    _entrypoint: &str,
    _args: CallArgs,
    _amount: Option<Balance>
) -> T {
    unimplemented!()
}

/// Set the caller address for the next [call_contract].
pub fn set_caller(_address: Address) {
    unimplemented!()
}

/// Returns predefined account.
pub fn get_account(_n: usize) -> Address {
    unimplemented!()
}

/// Return possible error, from the previous execution.
pub fn get_error() -> Option<OdraError> {
    unimplemented!()
}

/// Returns an event from the given contract.
pub fn get_event<T: OdraType + OdraEvent>(_address: Address, _index: i32) -> Result<T, EventError> {
    unimplemented!()
}

/// Increases the current value of block_time.
pub fn advance_block_time_by(_seconds: u64) {
    unimplemented!()
}

/// Returns the balance of the account associated with the given address.
pub fn token_balance(_address: Address) -> Balance {
    unimplemented!()
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> Balance {
    Balance::from(1_000_000_000)
}

/// Expects the `block` execution will fail with the specific error.
pub fn assert_exception<E, F>(_err: E, _block: F)
where
    E: Into<OdraError>,
    F: Fn() + std::panic::RefUnwindSafe
{
    unimplemented!()
}

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}

/// Returns last call gas cost.
pub fn last_call_contract_gas_cost() -> Balance {
    unimplemented!()
}
