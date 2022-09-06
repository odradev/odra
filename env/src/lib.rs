use odra_types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs};
pub mod unwrap_or_revert;

cfg_if::cfg_if! {
    if #[cfg(feature = "wasm")] {
        mod contract_env;
        pub use contract_env::ContractEnv;
    } else if #[cfg(feature = "mock-vm")] {
        pub use odra_mock_vm::{ContractEnv, TestEnv};
        pub mod test_utils;
    } else if #[cfg(feature = "wasm-test")] {
        mod test_env;
        mod mock_contract_env;
        pub mod test_utils;
        pub use mock_contract_env::ContractEnv;
        pub use test_env::TestEnv;
    }
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(address: &Address, entrypoint: &str, args: &RuntimeArgs) -> T
where
    T: CLTyped + FromBytes,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let has_return = odra_types::CLType::Unit != T::cl_type();
            let result = TestEnv::call_contract(address, entrypoint, args, has_return);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        } else if #[cfg(feature = "wasm-test")] {
            let has_return = types::CLType::Unit != T::cl_type();
            let result = TestEnv::call_contract(address, entrypoint, args, has_return);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        }  else if #[cfg(feature = "wasm")] {
            let res = ContractEnv::call_contract(address, entrypoint, args);
            let res = odra_types::bytesrepr::deserialize(res);
            unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(res)
        } else {
            compile_error!("Unknown feature")
        }
    }
}
