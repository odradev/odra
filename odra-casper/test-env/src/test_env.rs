//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
use crate::env::ENV;
use casper_types::account::{AccountHash, ACCOUNT_HASH_LENGTH};
use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_casper_types::{Address, Balance, CallArgs, OdraType};
use odra_types::{
    event::{EventError, OdraEvent},
    OdraError
};

/// Returns backend name.
pub fn backend_name() -> String {
    "CasperVM".to_string()
}

/// Deploy WASM file with arguments.
pub fn register_contract(name: &str, args: CallArgs) -> Address {
    ENV.with(|env| {
        let wasm_name = format!("{}.wasm", name);
        env.borrow_mut().deploy_contract(&wasm_name, args);
        let contract_package_hash = format!("{}_package_hash", name);
        let contract_package_hash = env
            .borrow()
            .contract_package_hash_from_name(&contract_package_hash);
        let default_account = env.borrow().get_account(0);
        env.borrow_mut().set_caller(default_account);
        contract_package_hash.try_into().unwrap()
    })
}

/// Call contract under a given address.
pub fn call_contract<T: OdraType>(
    addr: Address,
    entrypoint: &str,
    args: CallArgs,
    amount: Option<Balance>
) -> T {
    ENV.with(|env| {
        let contract_package_hash = addr.as_contract_package_hash().unwrap();
        if let Some(amount) = amount {
            env.borrow_mut().attach_value(amount.inner());
        }
        env.borrow_mut()
            .call_contract(*contract_package_hash, entrypoint, args)
    })
}

/// Set the caller address for the next [call_contract].
pub fn set_caller(address: Address) {
    ENV.with(|env| env.borrow_mut().set_caller(address))
}

/// Returns predefined account.
pub fn get_account(n: usize) -> Address {
    ENV.with(|env| env.borrow().get_account(n))
}

/// Return possible error, from the previous execution.
pub fn get_error() -> Option<OdraError> {
    ENV.with(|env| env.borrow().get_error())
}

/// Returns an event from the given contract.
pub fn get_event<T: OdraType + OdraEvent>(address: Address, index: i32) -> Result<T, EventError> {
    ENV.with(|env| env.borrow().get_event(address, index))
}

/// Increases the current value of block_time.
pub fn advance_block_time_by(seconds: u64) {
    ENV.with(|env| env.borrow_mut().advance_block_time_by(seconds))
}

/// Returns the balance of the account associated with the given address.
pub fn token_balance(address: Address) -> Balance {
    ENV.with(|env| env.borrow().token_balance(address).into())
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> Balance {
    Balance::from(1_000_000_000)
}

/// Returns zero address for an account.
pub fn zero_address() -> Address {
    Address::Account(AccountHash::new([0; ACCOUNT_HASH_LENGTH]))
}

/// Expects the `block` execution will fail with the specific error.
pub fn assert_exception<E, F>(err: E, block: F)
where
    E: Into<OdraError>,
    F: Fn() + std::panic::RefUnwindSafe
{
    let _ = std::panic::catch_unwind(|| {
        block();
    });

    let expected: OdraError = err.into();
    let msg = format!("Expected {:?} error.", expected);
    let error: OdraError = get_error().expect(&msg);
    assert_eq!(error, expected);
}

/// Returns CSPR token metadata.
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Returns last call gas cost.
pub fn last_call_contract_gas_cost() -> Balance {
    ENV.with(|env| env.borrow().last_call_contract_gas_cost().into())
}

/// Returns the amount of gas paid for last call.
pub fn last_call_contract_gas_used() -> Balance {
    ENV.with(|env| env.borrow().last_call_contract_gas_used().into())
}
