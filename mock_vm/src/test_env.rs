use std::collections::HashMap;

use odra_types::{bytesrepr::Bytes, Address, OdraError, RuntimeArgs};

use crate::{borrow_env, EntrypointCall, mock_vm::default_accounts};

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
        F: Fn() -> (),
        E: Into<OdraError>,
    {
        block();
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
}
