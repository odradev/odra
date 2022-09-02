use std::collections::HashMap;

use odra_types::{bytesrepr::Bytes, event::EventError, Address, EventData, OdraError, RuntimeArgs};

use crate::{mock_vm::default_accounts, EntrypointCall};

macro_rules! delegate_to_env {
    (
        $(
            $(#[$outer:meta])*
            $func_name:ident($( $param_ident:ident : $param_ty:ty ),*) $( -> $ret:ty)*
        )+
    ) => {
        $(
            $(#[$outer])*
            pub fn $func_name( $($param_ident : $param_ty),* ) $(-> $ret)* {
                crate::borrow_env().$func_name($($param_ident),*)
            }
        )+
    }
}

/// Describes test environment API. TestEnv delegates methods to the underlying env implementation.
///
/// Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
pub struct TestEnv;

impl TestEnv {
    delegate_to_env! {
        /// Registers the contract in the test environment.
        register_contract(
            constructor: Option<(String, RuntimeArgs, EntrypointCall)>,
            constructors: HashMap<String, EntrypointCall>,
            entrypoints: HashMap<String, EntrypointCall>
        ) -> Address
        /// Calls contract at `address` invoking the `entrypoint` with `args`.
        ///
        /// Returns optional raw bytes to further processing.
        call_contract(
            address: &Address,
            entrypoint: &str,
            args: &RuntimeArgs,
            has_return: bool
        ) -> Option<Bytes>
        /// Increases the current value of block_time.
        advance_block_time_by(seconds: u64)
        /// Returns the backend name.
        get_backend_name() -> String
        /// Replaces the current caller.
        set_caller(address: &Address)
        /// Gets nth event emitted by the contract at `address`.
        get_event(address: &Address, index: i32) -> Result<EventData, EventError>
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

        let exec_err = crate::borrow_env()
            .error()
            .expect("An error expected, but did not occur");
        assert_eq!(exec_err, err.into());
    }

    /// Returns nth test user account.
    pub fn get_account(n: usize) -> Address {
        *default_accounts().get(n).unwrap()
    }
}
