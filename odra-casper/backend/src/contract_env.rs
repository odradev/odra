//! Casper backend for WASM.
//!
//! It provides all the required functions to communicate between Odra and Casper.
use alloc::vec::Vec;
use casper_contract::contract_api::runtime;
use casper_contract::{
    contract_api::system::transfer_from_purse_to_account, unwrap_or_revert::UnwrapOrRevert
};
use core::ops::Deref;
use odra_casper_shared::native_token::NativeTokenMetadata;
use odra_core::event::OdraEvent;
use odra_types::casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use odra_types::casper_types::{crypto, CLTyped, RuntimeArgs, U512};
use odra_types::ExecutionError;
use odra_types::{Address, BlockTime};

use crate::{casper_env, utils::get_or_create_main_purse};

pub(crate) static mut ATTACHED_VALUE: U512 = U512::zero();

/// Returns blocktime.
#[inline(always)]
pub fn get_block_time() -> BlockTime {
    casper_env::get_block_time()
}

/// Returns contract caller.
#[inline(always)]
pub fn caller() -> Address {
    casper_env::caller()
}

/// Returns current contract address.
#[inline(always)]
pub fn self_address() -> Address {
    casper_env::self_address()
}

/// Store a value into the storage.
#[inline(always)]
pub fn set_var<T: CLTyped + ToBytes>(key: &[u8], value: T) {
    casper_env::set_key(key, value);
}

/// Read value from the storage.
#[inline(always)]
pub fn get_var<T: FromBytes>(key: &[u8]) -> Option<T> {
    casper_env::get_key(key)
}

/// Store the mapping value under a given key.
#[inline(always)]
pub fn set_dict_value<K: ToBytes, V: CLTyped + ToBytes>(dict: &[u8], key: &K, value: V) {
    casper_env::set_dict_value(dict, key, value);
}

/// Read value from the mapping.
#[inline(always)]
pub fn get_dict_value<K: ToBytes, T: FromBytes>(dict: &[u8], key: &K) -> Option<T> {
    casper_env::get_dict_value(dict, key)
}

/// Revert the execution.
#[inline(always)]
pub fn revert<E>(error: E) -> !
where
    E: Into<ExecutionError>
{
    casper_env::revert(error.into().code());
}

/// Emits event.
#[inline(always)]
pub fn emit_event<T>(event: T)
where
    T: ToBytes + OdraEvent
{
    casper_env::emit_event(event);
}

/// Call another contract.
#[inline(always)]
pub fn call_contract<T: CLTyped + FromBytes>(
    address: Address,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>
) -> T {
    let contract_package_hash = *address.as_contract_package_hash().unwrap_or_revert();
    if let Some(amount) = amount {
        casper_env::call_contract_with_amount(
            contract_package_hash,
            entrypoint,
            args.deref().clone(),
            amount
        )
    } else {
        casper_env::call_contract(contract_package_hash, entrypoint, args.deref().clone())
    }
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
pub fn one_token() -> U512 {
    U512::from(1_000_000_000)
}

/// Returns the balance of the account associated with the currently executing contract.
pub fn self_balance() -> U512 {
    casper_env::self_balance()
}

/// Returns amount of native token attached to the call.
pub fn attached_value() -> U512 {
    unsafe { ATTACHED_VALUE }
}

/// Transfers native token from the contract caller to the given address.
pub fn transfer_tokens<B: Into<U512>>(to: &Address, amount: B) {
    let main_purse = get_or_create_main_purse();

    match to {
        Address::Account(account) => {
            transfer_from_purse_to_account(main_purse, *account, amount.into(), None)
                .unwrap_or_revert();
        }
        Address::Contract(_) => revert(ExecutionError::can_not_transfer_to_contract())
    };
}

/// Returns CSPR token metadata
pub fn native_token_metadata() -> NativeTokenMetadata {
    NativeTokenMetadata::new()
}

/// Verifies the signature created using the Backend's default signature scheme.
pub fn verify_signature(
    message: &Bytes,
    signature: &Bytes,
    public_key: &crypto::PublicKey
) -> bool {
    let (signature, _) = crypto::Signature::from_bytes(signature.as_slice()).unwrap_or_revert();
    crypto::verify(message.as_slice(), &signature, public_key).is_ok()
}

/// Creates a hash of the given input. Uses default hash for given backend.
pub fn hash<T: AsRef<[u8]>>(input: T) -> Vec<u8> {
    runtime::blake2b(input).to_vec()
}
