//! Describes test environment API. Delegates methods to the underlying env implementation.
//!
//! Depending on the selected feature, the actual test env is dynamically loaded in the runtime or the Odra local MockVM is used.
use odra_core::event::{EventError, OdraEvent};
use odra_types::{
    casper_types::{
        bytesrepr::{Bytes, FromBytes, ToBytes},
        RuntimeArgs, U512
    },
    Address, BlockTime, PublicKey
};
use odra_types::{OdraAddress, OdraError};
use std::{collections::BTreeMap, panic::AssertUnwindSafe};

use crate::{native_token::NativeTokenMetadata, EntrypointArgs, EntrypointCall};

macro_rules! delegate_to_env {
    (
        $(
            $(#[$outer:meta])*
            fn $func_name:ident($( $param_ident:ident : $param_ty:ty ),*) $( -> $ret:ty)*
        )+
    ) => {
        $(
            $(#[$outer])*
            pub fn $func_name( $($param_ident : $param_ty),* ) $(-> $ret)* {
                crate::borrow_env().$func_name($($param_ident),*)
            }
        )+
    }
}

delegate_to_env! {
    /// Registers the contract in the test environment.
    fn register_contract(
        constructor: Option<(String, &RuntimeArgs, EntrypointCall)>,
        constructors: BTreeMap<String, (EntrypointArgs, EntrypointCall)>,
        entrypoints: BTreeMap<String, (EntrypointArgs, EntrypointCall)>
    ) -> Address
    /// Increases the current value of block_time.
    fn advance_block_time_by(milliseconds: BlockTime)
    /// Returns the backend name.
    fn get_backend_name() -> String
    /// Replaces the current caller.
    fn set_caller(address: Address)
    /// Returns the balance of the account associated with the given address.
    fn token_balance(address: Address) -> U512
    /// Returns nth test user account.
    fn get_account(n: usize) -> Address
}

/// Expects the `block` execution will fail with the specific error.
pub fn assert_exception<F, E>(err: E, block: F)
where
    F: FnOnce(),
    E: Into<OdraError>
{
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        block();
    }));
    let exec_err = crate::borrow_env()
        .error()
        .expect("An error expected, but did not occur");

    assert_eq!(exec_err, err.into());
}

/// Returns the value that represents one native token.
pub fn one_token() -> U512 {
    U512::one()
}

/// Calls contract at `address` invoking the `entrypoint` with `args`.
///
/// Returns optional raw bytes to further processing.
pub fn call_contract<T: ToBytes + FromBytes>(
    address: Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>
) -> T {
    crate::borrow_env().call_contract(address, entrypoint, args, amount)
}

/// Gets nth event emitted by the contract at `address`.
pub fn get_event<T: FromBytes + OdraEvent>(address: Address, index: i32) -> Result<T, EventError> {
    let bytes = crate::borrow_env().get_event(address, index);

    bytes.and_then(|bytes| {
        let event_name = extract_event_name(&bytes)?;
        if event_name == format!("event_{}", T::name()) {
            T::from_bytes(&bytes)
                .map_err(|_| EventError::Parsing)
                .map(|r| r.0)
        } else {
            Err(EventError::UnexpectedType(event_name))
        }
    })
}

/// Returns the platform native token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Returns last call gas cost.
pub fn last_call_contract_gas_cost() -> U512 {
    U512::zero()
}

/// Returns the amount of gas paid for last call.
pub fn last_call_contract_gas_used() -> U512 {
    U512::zero()
}

/// Returns the total amount of gas used by the address.
/// Currently MockVM doesn't charge gas.
pub fn total_gas_used(address: Address) -> U512 {
    if address.is_contract() {
        panic!("Contract {:?} can't burn gas.", address)
    }
    U512::zero()
}

/// Returns the report of entrypoints called, contract deployed and gas used.
/// Currently MockVM doesn't charge gas.
pub fn gas_report() -> Vec<(String, U512)> {
    Vec::new()
}

/// Returns the name of the passed event
fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
    let name = FromBytes::from_bytes(bytes).map_err(|_| EventError::Formatting)?;
    Ok(name.0)
}

/// Signs the message using the private key associated with the given address.
pub fn sign_message(message: &Bytes, address: &Address) -> Bytes {
    let public_key = public_key(address);
    let mut message = message.inner_bytes().clone();
    message.extend(public_key.into_bytes().unwrap());
    Bytes::from(message)
}

/// Returns the public key of the account associated with the given address.
pub fn public_key(address: &Address) -> PublicKey {
    crate::borrow_env().public_key(address)
}
