use std::collections::HashMap;

use odra_types::{RuntimeArgs, Address, bytesrepr::Bytes, OdraError};

use crate::{EntrypointCall, borrow_env};

pub struct TestEnv;

impl TestEnv {
    pub fn register_contract(
        entrypoints: HashMap<String, EntrypointCall>,
        args: RuntimeArgs,
    ) -> Address {
        borrow_env().register_contract(entrypoints, args)
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

    pub fn backend_name() -> String {
        borrow_env().get_backend_name()
    }
}
