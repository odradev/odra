//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
//!
use crate::cosmos_test_env::ENV;
use odra_cosmos_shared::native_token::NativeTokenMetadata;
use odra_cosmos_types::{Address, Balance, CallArgs, OdraType};
use odra_types::{
    event::{EventError, OdraEvent},
    OdraError
};

/// Returns backend name.
pub fn backend_name() -> String {
    "Cosmwasm".to_string()
}

/// Deploy WASM file with arguments.
pub fn register_contract(name: &str, args: CallArgs) -> Address {
    ENV.with(|env| {
        let wasm_name = format!("{}.wasm", name);
        env.borrow_mut().deploy_contract(&wasm_name, "init", args);

        Address::new(&[])
    })
}

/// Call contract under a given address.
pub fn call_contract<T: OdraType>(
    _addr: Address,
    entrypoint: &str,
    args: CallArgs,
    amount: Option<Balance>
) -> T {
    ENV.with(|env| {
        // vec![110, 117, 108, 108] == null == ()
        if let Ok(null) = T::deser(vec![110, 117, 108, 108]) {
            if let Some(amount) = amount {
                env.borrow_mut().attach_value(amount);
            }

            env.borrow_mut().execute(entrypoint, args);
            null
        } else {
            env.borrow_mut().query(entrypoint, args)
        }
    })
}

/// Set the caller address for the next [call_contract].
pub fn set_caller(address: Address) {
    ENV.with(|env| {
        env.borrow_mut().set_caller(address);
    });
}

/// Returns predefined account.
pub fn get_account(_n: usize) -> Address {
    unimplemented!()
}

/// Return possible error, from the previous execution.
pub fn get_error() -> Option<OdraError> {
    ENV.with(|env| env.borrow().get_error())
}

/// Returns an event from the given contract.
pub fn get_event<T: OdraType + OdraEvent>(_address: Address, _index: i32) -> Result<T, EventError> {
    unimplemented!()
}

/// Increases the current value of block_time.
pub fn advance_block_time_by(seconds: u64) {
    ENV.with(|env| env.borrow_mut().advance_block_time_by(seconds))
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

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}
