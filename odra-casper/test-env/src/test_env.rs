//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
use std::panic::AssertUnwindSafe;

use crate::env::ENV;
use odra_casper_shared::{consts, native_token::NativeTokenMetadata};
use odra_types::{
    Address,
    event::{EventError, OdraEvent},
    OdraError, casper_types::{RuntimeArgs, bytesrepr::{FromBytes, Bytes, ToBytes}, account::{AccountHash, ACCOUNT_HASH_LENGTH}, CLTyped, U512}, PublicKey
};

/// Returns backend name.
pub fn backend_name() -> String {
    "CasperVM".to_string()
}

/// Deploy WASM file with arguments.
pub fn register_contract(name: &str, args: &RuntimeArgs, constructor_name: Option<String>) -> Address {
    ENV.with(|env| {
        let wasm_name = format!("{}.wasm", name);
        let package_hash_key_name = format!("{}_package_hash", name);
        let mut args = args.clone();
        args.insert(
            consts::PACKAGE_HASH_KEY_NAME_ARG,
            package_hash_key_name.clone()
        ).unwrap();
        args.insert(consts::ALLOW_KEY_OVERRIDE_ARG, true).unwrap();
        args.insert(consts::IS_UPGRADABLE_ARG, false).unwrap();

        if let Some(constructor_name) = constructor_name {
            args.insert(consts::CONSTRUCTOR_NAME_ARG, constructor_name).unwrap();
        };

        env.borrow_mut().deploy_contract(&wasm_name, &args);
        let contract_package_hash = env
            .borrow()
            .contract_package_hash_from_name(&package_hash_key_name);
        contract_package_hash.try_into().unwrap()
    })
}

/// Call contract under a given address.
pub fn call_contract<T: CLTyped + ToBytes + FromBytes>(
    addr: Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>
) -> T {
    ENV.with(|env| {
        let contract_package_hash = addr.as_contract_package_hash().unwrap();
        if let Some(amount) = amount {
            env.borrow_mut().attach_value(amount);
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
pub fn get_event<T: FromBytes + OdraEvent>(address: Address, index: i32) -> Result<T, EventError> {
    ENV.with(|env| env.borrow().get_event(address, index))
}

/// Increases the current value of block_time.
pub fn advance_block_time_by(milliseconds: u64) {
    ENV.with(|env| env.borrow_mut().advance_block_time_by(milliseconds))
}

/// Returns the balance of the account associated with the given address.
pub fn token_balance(address: Address) -> U512 {
    ENV.with(|env| env.borrow().token_balance(address).into())
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> U512 {
    U512::from(1_000_000_000)
}

/// Returns zero address for an account.
pub fn zero_address() -> Address {
    Address::Account(AccountHash::new([0; ACCOUNT_HASH_LENGTH]))
}

/// Expects the `block` execution will fail with the specific error.
pub fn assert_exception<E, F>(err: E, block: F)
where
    E: Into<OdraError>,
    F: FnOnce()
{
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        block();
    }));

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
pub fn last_call_contract_gas_cost() -> U512 {
    ENV.with(|env| env.borrow().last_call_contract_gas_cost().into())
}

/// Returns the amount of gas paid for last call.
pub fn last_call_contract_gas_used() -> U512 {
    ENV.with(|env| env.borrow().last_call_contract_gas_used().into())
}

/// Returns total gas used by the account.
pub fn total_gas_used(address: Address) -> U512 {
    ENV.with(|env| env.borrow().total_gas_used(address).into())
}

/// Returns the report of entrypoints called, contract deployed and gas used.
pub fn gas_report() -> Vec<(String, U512)> {
    let mut report = Vec::new();
    ENV.with(|env| {
        let env = env.borrow();
        for (reason, gas) in env.gas_report() {
            report.push((reason, gas.into()));
        }
    });

    report
}

/// Signs the message using the private key associated with the given address.
pub fn sign_message(message: &Bytes, address: &Address) -> Bytes {
    ENV.with(|env| env.borrow().sign_message(message, address))
}

/// Returns the public key of the account associated with the given address.
pub fn public_key(address: &Address) -> PublicKey {
    ENV.with(|env| env.borrow().public_key(address))
}
