//! Casper backend for Livenet.
//!
//! It provides all the required functions to communicate between Odra and Casper Livenets.

use crate::casper_client::LivenetKeyMaker;
use odra_casper_shared::key_maker::KeyMaker;
use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_types::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        U512
    },
    Address, ExecutionError, PublicKey
};

use crate::client_env;

pub fn self_address() -> Address {
    unimplemented!()
}

pub fn caller() -> Address {
    unimplemented!()
}

pub fn set_var<T: ToBytes>(_: &[u8], _: T) {
    unimplemented!()
}

pub fn get_var<T: FromBytes>(key: &[u8]) -> Option<T> {
    client_env::get_var_from_current_contract(key)
}

pub fn set_dict_value<K: ToBytes, V: ToBytes>(_: &[u8], _: &K, _: V) {
    unimplemented!()
}

pub fn get_dict_value<K: ToBytes, T: FromBytes>(seed: &[u8], key: &K) -> Option<T> {
    client_env::get_dict_value_from_current_contract(seed, key)
}

pub fn emit_event<T>(_: T)
where
    T: ToBytes {
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

pub fn attached_value() -> U512 {
    unimplemented!()
}

pub fn one_token() -> U512 {
    unimplemented!()
}

pub fn transfer_tokens<B: Into<U512>>(_: &Address, _: B) {
    unimplemented!()
}

pub fn self_balance() -> U512 {
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
