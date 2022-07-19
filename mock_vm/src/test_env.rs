use std::collections::HashMap;

use odra_types::{
    bytesrepr::Bytes, event::Error as EventError, Address, EventData, OdraError, RuntimeArgs,
};

use crate::{borrow_env, mock_vm::default_accounts, EntrypointCall};

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(
        constructor: Option<(String, RuntimeArgs, EntrypointCall)>,
        entrypoints: HashMap<String, EntrypointCall>,
    ) -> Address {
        borrow_env().register_contract(constructor, entrypoints)
    }

    pub fn call_contract(
        address: &Address,
        entrypoint: &str,
        args: &RuntimeArgs,
        has_return: bool,
    ) -> Option<Bytes> {
        borrow_env().call_contract(address, entrypoint, args, has_return)
    }

    pub fn assert_exception<F, E>(err: E, block: F)
    where
        F: Fn() -> () + std::panic::RefUnwindSafe,
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

    pub fn backend_name() -> String {
        borrow_env().get_backend_name()
    }

    pub fn set_caller(address: &Address) {
        borrow_env().set_caller(address)
    }

    pub fn get_account(n: usize) -> Address {
        *default_accounts().get(n).unwrap()
    }

    pub fn get_event(address: &Address, index: i32) -> Result<EventData, EventError> {
        borrow_env().get_event(address, index)
    }
}
