use std::collections::HashMap;

use odra_types::{
    bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs, U256,
};

use crate::{borrow_env, EntrypointCall};

/// Describes test environment API. TestEnv delegates methods to the underlying env implementation.
///
/// Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
pub struct TestEnv;

impl TestEnv {
    /// Registers the contract in the test environment.
    pub fn register_contract(
        constructor: Option<(String, RuntimeArgs, EntrypointCall)>,
        constructors: HashMap<String, EntrypointCall>,
        entrypoints: HashMap<String, EntrypointCall>,
    ) -> Address {
        borrow_env().register_contract(constructor, constructors, entrypoints)
    }

    /// Calls contract at `address` invoking the `entrypoint` with `args`.
    ///
    /// Returns optional raw bytes to further processing.
    pub fn call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        _has_return: bool,
    ) -> Option<Bytes> {
        borrow_env().call_contract(address, entrypoint, args)
    }

    /// Expects the `block` execution will fail with the specific error.
    pub fn assert_exception<F, E>(err: E, block: F)
    where
        F: Fn() + std::panic::RefUnwindSafe,
        E: Into<OdraError>,
    {
        let _ = std::panic::catch_unwind(|| {
            block();
        });

        let exec_err = borrow_env()
            .error()
            .expect("An error expected, but did not occur");
        assert_eq!(exec_err, err.into());
    }

    /// Returns the backend name.
    pub fn backend_name() -> String {
        borrow_env().get_backend_name()
    }

    /// Replaces the current caller.
    pub fn set_caller(address: &Address) {
        borrow_env().set_caller(address)
    }

    /// Returns nth test user account.
    pub fn get_account(n: usize) -> Address {
        borrow_env().get_address(n)
    }

    pub fn get_balance(address: Address) -> U256 {
        borrow_env().get_balance(address)
    }

    /// Gets nth event emitted by the contract at `address`.
    pub fn get_event(address: &Address, index: i32) -> Result<EventData, EventError> {
        borrow_env().get_event(address, index)
    }
}
