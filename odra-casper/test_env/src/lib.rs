//! TestEnv bindings for Casper's InMemoryWasmTestBuilder.

use crate::env::ENV;
use casper_types::{bytesrepr::Bytes, RuntimeArgs};
use odra::types::{event::EventError, Address as OdraAddress, EventData, OdraError, U512};
use odra_casper_shared::casper_address::CasperAddress;

pub mod env;

/// Returns backend name.
#[no_mangle]
pub fn backend_name() -> String {
    "CasperVM".to_string()
}

/// Deploy WASM file with arguments.
#[no_mangle]
pub fn register_contract(name: &str, args: &RuntimeArgs) -> OdraAddress {
    ENV.with(|env| {
        let wasm_name = format!("{}.wasm", name);
        env.borrow_mut().deploy_contract(&wasm_name, args.clone());

        let contract_package_hash = format!("{}_package_hash", name);
        let contract_package_hash = env
            .borrow()
            .contract_package_hash_from_name(&contract_package_hash);
        let casper_address: CasperAddress = contract_package_hash.into();
        OdraAddress::try_from(casper_address).unwrap()
    })
}

/// Call contract under a given address.
#[no_mangle]
pub fn call_contract(
    addr: &OdraAddress,
    entrypoint: &str,
    args: &RuntimeArgs,
    has_return: bool,
    amount: Option<U512>,
) -> Option<Bytes> {
    ENV.with(|env| {
        let casper_address = CasperAddress::try_from(*addr).unwrap();
        let contract_package_hash = casper_address.as_contract_package_hash().unwrap();
        if let Some(amount) = amount {
            env.borrow_mut().attach_value(amount);
        }
        env.borrow_mut().call_contract(
            *contract_package_hash,
            entrypoint,
            args.to_owned(),
            has_return,
        )
    })
}

/// Set the caller address for the next [call_contract].
#[no_mangle]
pub fn set_caller(address: &OdraAddress) {
    ENV.with(|env| {
        let casper_address = CasperAddress::try_from(*address).unwrap();
        env.borrow_mut().set_caller(casper_address)
    })
}

/// Returns predefined account.
#[no_mangle]
pub fn get_account(n: usize) -> OdraAddress {
    ENV.with(|env| OdraAddress::try_from(env.borrow().get_account(n)).unwrap())
}

/// Return possible error, from the previous execution.
#[no_mangle]
pub fn get_error() -> Option<OdraError> {
    ENV.with(|env| env.borrow().get_error())
}

/// Returns an event from the given contract.
#[no_mangle]
pub fn get_event(address: &OdraAddress, index: i32) -> Result<EventData, EventError> {
    ENV.with(|env| {
        let casper_address = CasperAddress::try_from(*address).unwrap();
        env.borrow().get_event(casper_address, index)
    })
}

/// Increases the current value of block_time.
#[no_mangle]
fn advance_block_time_by(seconds: u64) {
    ENV.with(|env| env.borrow_mut().advance_block_time_by(seconds))
}

/// Returns the balance of the account associated with the given address.
#[no_mangle]
pub fn token_balance(address: OdraAddress) -> U512 {
    let casper_address = CasperAddress::try_from(address).unwrap();
    ENV.with(|env| env.borrow().token_balance(casper_address))
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
#[no_mangle]
pub fn one_token() -> U512 {
    U512::from(1_000_000_000)
}
