pub mod contract_def;
mod instance;
mod list;
mod mapping;
mod unwrap_or_revert;
mod variable;

use std::fmt::Debug;
use types::{bytesrepr::FromBytes, Address, CLTyped, RuntimeArgs, U512};

pub use {
    instance::Instance,
    list::List,
    mapping::Mapping,
    odra_proc_macros::{execution_error, external_contract, module, odra_error, Event, Instance},
    odra_types as types, odra_utils as utils,
    unwrap_or_revert::UnwrapOrRevert,
    variable::Variable,
};


#[cfg(test)]
pub mod test_utils;

#[cfg(feature = "mock-vm")]
pub use odra_mock_vm::{test_env, contract_env};
#[cfg(feature = "casper")]
pub use odra_casper_backend::contract_env;
#[cfg(feature = "casper-test")]
pub use odra_casper_test_env::test_env;
#[cfg(feature = "casper-test")]
pub mod contract_env {
    use odra_types::CLTyped;
    use odra_types::CLValue;
    use odra_types::ExecutionError;
    use odra_types::U512;
    use odra_types::Address;
    use odra_types::bytesrepr::FromBytes;
    use odra_types::bytesrepr::ToBytes;
    use odra_types::event::Event;
    
    pub fn self_address() -> Address {
        unimplemented!()
    }
    
    pub fn caller() -> Address {
        unimplemented!()
    }
    
    pub fn set_var<T: CLTyped + ToBytes>(key: &str, value: T) {
        unimplemented!()
    }
    
    pub fn get_var(key: &str) -> Option<CLValue> {
        unimplemented!()
    }
    
    pub fn set_dict_value<K: ToBytes, V: ToBytes + FromBytes + CLTyped>(
        dict: &str,
        key: &K,
        value: V,
    ) {
        unimplemented!()
    }
    
    pub fn get_dict_value<K: ToBytes>(dict: &str, key: &K) -> Option<CLValue> {
        unimplemented!()
    }
    
    pub fn emit_event<T>(event: &T)
    where
        T: ToBytes + Event,
    {
        unimplemented!()
    }
    
    pub fn revert<E>(error: E) -> !
    where
        E: Into<ExecutionError>,
    {
        unimplemented!()
    }
    
    pub fn get_block_time() -> u64 {
        unimplemented!()
    }
    
    pub fn attached_value() -> U512 {
        unimplemented!()
    }
    
    pub fn one_token() -> U512 {
        unimplemented!()
    }
    
    pub fn transfer_tokens(to: Address, amount: U512) -> bool {
        unimplemented!()
    }
    
    pub fn self_balance() -> U512 {
        unimplemented!()
    }
    
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns already parsed result.
pub fn call_contract<T>(
    address: &Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>,
) -> T
where
    T: CLTyped + FromBytes + Debug,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "mock-vm")] {
            let result = test_env::call_contract(address, entrypoint, args, amount);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        } else if #[cfg(feature = "casper-test")] {
            let has_return = types::CLType::Unit != T::cl_type();
            let result = test_env::call_contract(address, entrypoint, args, has_return, amount);
            match result {
                Some(bytes) => T::from_bytes(bytes.as_slice()).unwrap().0,
                None => T::from_bytes(&[]).unwrap().0,
            }
        }  else if #[cfg(feature = "casper")] {
            let res = contract_env::call_contract(address, entrypoint, args, amount);
            types::bytesrepr::deserialize(res).unwrap_or_revert()
        } else {
            compile_error!("Unknown feature")
        }
    }
}
