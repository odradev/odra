//! Exposes the public API to communicate with the host.

use core::panic;
use odra_core::event::OdraEvent;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use odra_types::casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use odra_types::casper_types::{CLTyped, U512};
use odra_types::{Address, BlockTime, PublicKey};
use odra_types::{ExecutionError, OdraError};

use crate::{borrow_env, debug, native_token::NativeTokenMetadata};

/// Returns the current block time.
pub fn get_block_time() -> BlockTime {
    borrow_env().get_block_time()
}

/// Gets the address of the currently executing contract.
pub fn caller() -> Address {
    borrow_env().caller()
}

/// Returns the address of currently executing contract.
pub fn self_address() -> Address {
    borrow_env().callee()
}

/// Stores the `value` under `key`.
pub fn set_var<T: ToBytes + CLTyped>(key: &[u8], value: T) {
    borrow_env().set_var(key, value)
}

/// Gets a value stored under `key`.
pub fn get_var<T: FromBytes>(key: &[u8]) -> Option<T> {
    borrow_env().get_var(key)
}

/// Puts a key-value into a collection.
pub fn set_dict_value<K: ToBytes, V: ToBytes + CLTyped>(dict: &[u8], key: &K, value: V) {
    borrow_env().set_dict_value(dict, key.to_bytes().unwrap().as_slice(), value)
}

/// Gets the value from the `dict` collection under `key`.
pub fn get_dict_value<K: ToBytes, T: FromBytes>(dict: &[u8], key: &K) -> Option<T> {
    let key = key.to_bytes().unwrap();
    let key = key.as_slice();
    borrow_env().get_dict_value(dict, key)
}

/// Stops execution of a contract and reverts execution effects with a given [`ExecutionError`].
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>
{
    let execution_error: ExecutionError = error.into();
    let odra_error: OdraError = execution_error.clone().into();
    let callstack_tip = borrow_env().callstack_tip();

    borrow_env().revert(odra_error);

    std::panic::set_hook(Box::new(|info| {
        let backtrace = Backtrace::capture();
        if matches!(backtrace.status(), BacktraceStatus::Captured) {
            debug::print_first_n_frames(&backtrace, 30);
        }
        debug::print_panic_error(info);
    }));
    panic!(
        "{}",
        debug::format_panic_message(&execution_error, &callstack_tip)
    );
}

/// Sends an event to the execution environment.
pub fn emit_event<T: ToBytes + OdraEvent>(event: T) {
    let event_data = event.to_bytes().unwrap();
    borrow_env().emit_event(&event_data);
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> U512 {
    borrow_env().attached_value()
}

/// Returns the value that represents one native token.
pub fn one_token() -> U512 {
    U512::one()
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens<B: Into<U512>>(to: &Address, amount: B) {
    let callee = borrow_env().callee();
    let amount = amount.into();
    borrow_env().transfer_tokens(&callee, to, &amount);
}

/// Returns the balance of the account associated with the current contract.
pub fn self_balance() -> U512 {
    borrow_env().self_balance()
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Verifies the signature created using the Backend's default signature scheme.
pub fn verify_signature(message: &Bytes, signature: &Bytes, public_key: &PublicKey) -> bool {
    let mut message = message.inner_bytes().clone();
    message.extend(public_key.into_bytes().unwrap());
    let mock_signature_bytes = Bytes::from(message);
    mock_signature_bytes == *signature
}

/// Creates a hash of the given input. Uses default hash for given backend.
pub fn hash<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    let mut s = DefaultHasher::new();
    input.as_ref().hash(&mut s);
    let hash = s.finish();
    hash.to_le_bytes().to_vec()
}
