use std::collections::HashMap;

use odra_types::{RuntimeArgs, Address, bytesrepr::Bytes, OdraError};

use crate::{EntrypointCall, borrow_env};

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

    pub fn execution_result() -> Result<(), OdraError> {
        match borrow_env().error() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub fn expect_revert<T: std::any::Any>(_: T, err: OdraError) {
        let exec_err = borrow_env()
            .error()
            .expect("An error expected, but the call succeeded");
        assert_eq!(exec_err, err);
    }

    pub fn backend_name() -> String {
        borrow_env().get_backend_name()
    }
}
