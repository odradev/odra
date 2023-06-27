//! Exposes the public API to communicate with the host.

use core::panic;
use std::backtrace::{Backtrace, BacktraceStatus};

use odra_mock_vm_types::{
    Address, Balance, BlockTime, Bytes, MockDeserializable, MockSerializable, OdraType, PublicKey
};
use odra_types::{event::OdraEvent, ExecutionError, OdraError};

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
pub fn set_var<T: MockSerializable + MockDeserializable>(key: &[u8], value: T) {
    borrow_env().set_var(key, value)
}

/// Gets a value stored under `key`.
pub fn get_var<T: OdraType>(key: &[u8]) -> Option<T> {
    borrow_env().get_var(key)
}

/// Puts a key-value into a collection.
pub fn set_dict_value<
    K: MockSerializable + MockDeserializable,
    V: MockSerializable + MockDeserializable
>(
    dict: &[u8],
    key: &K,
    value: V
) {
    borrow_env().set_dict_value(dict, key.ser().unwrap().as_slice(), value)
}

/// Gets the value from the `dict` collection under `key`.
pub fn get_dict_value<
    K: MockSerializable + MockDeserializable,
    T: MockSerializable + MockDeserializable
>(
    dict: &[u8],
    key: &K
) -> Option<T> {
    let key = key.ser().unwrap();
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
pub fn emit_event<T: OdraType + OdraEvent>(event: T) {
    let event_data = event.ser().unwrap();
    borrow_env().emit_event(&event_data);
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> Balance {
    borrow_env().attached_value()
}

/// Returns the value that represents one native token.
pub fn one_token() -> Balance {
    Balance::one()
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens<B: Into<Balance>>(to: &Address, amount: B) {
    let callee = borrow_env().callee();
    let amount = amount.into();
    borrow_env().transfer_tokens(&callee, to, &amount);
}

/// Returns the balance of the account associated with the current contract.
pub fn self_balance() -> Balance {
    borrow_env().self_balance()
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Verifies the signature created using the Backend's default signature scheme.
pub fn verify_signature(message: &Bytes, signature: &Bytes, public_key: &PublicKey) -> bool {
    let mut message = message.inner_bytes().clone();
    message.extend_from_slice(public_key.inner_bytes());
    let mock_signature_bytes = Bytes::from(message);
    mock_signature_bytes == *signature
}
