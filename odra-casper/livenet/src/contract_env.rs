//! Casper backend for Livenet.
//!
//! It provides all the required functions to communicate between Odra and Casper Livenets.

use crate::casper_client::LivenetKeyMaker;
use casper_types::bytesrepr::Bytes;
use casper_types::PublicKey;
use odra_casper_shared::key_maker::KeyMaker;
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

pub fn get_dict_value<K: OdraType, T: OdraType>(seed: &str, key: &K) -> Option<T> {
    client_env::get_dict_value_from_current_contract(seed, key)
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

pub fn transfer_tokens<B: Into<Balance>>(_: &Address, _: B) {
    unimplemented!()
}

pub fn self_balance() -> Balance {
    unimplemented!()
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    unimplemented!()
}

/// Verifies the signature created using the Backend's default signature scheme.
pub fn verify_signature(_: &Bytes, _: &Bytes, _: &PublicKey) -> bool {
    unimplemented!()
}

/// Creates a hash of the given input. Uses default hash for given backend.
pub fn hash<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    LivenetKeyMaker::blake2b(input.as_ref()).to_vec()
}
