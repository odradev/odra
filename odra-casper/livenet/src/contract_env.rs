use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_casper_types::{Address, Balance, BlockTime, OdraType};
use odra_types::{event::OdraEvent, ExecutionError};

use crate::client_env;

pub fn self_address() -> Address {
    unimplemented!()
}

pub fn caller() -> Address {
    unimplemented!()
}

pub fn set_var<T: OdraType>(_: &str, _: T) {
    unimplemented!()
}

pub fn get_var<T: OdraType>(key: &str) -> Option<T> {
    client_env::get_var_from_current_contract(key)
}

pub fn set_dict_value<K: OdraType, V: OdraType>(_: &str, _: &K, _: V) {
    unimplemented!()
}

pub fn get_dict_value<K: OdraType, T: OdraType>(_: &str, _: &K) -> Option<T> {
    unimplemented!()
}

pub fn emit_event<T>(_: T)
where
    T: OdraType + OdraEvent
{
    unimplemented!()
}

pub fn revert<E>(_: E) -> !
where
    E: Into<ExecutionError>
{
    unimplemented!()
}

pub fn get_block_time() -> BlockTime {
    unimplemented!()
}

pub fn attached_value() -> Balance {
    unimplemented!()
}

pub fn one_token() -> Balance {
    unimplemented!()
}

pub fn transfer_tokens<B: Into<Balance>>(_: Address, _: B) -> bool {
    unimplemented!()
}

pub fn self_balance() -> Balance {
    unimplemented!()
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}
