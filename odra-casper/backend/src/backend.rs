//! Casper backend for WASM.
//!
//! It provides all the required functions to communicate between Odra and Casper.

pub use casper_contract::{
    self,
    contract_api::{runtime, storage},
};
use casper_contract::{
    contract_api::system::transfer_from_purse_to_account, unwrap_or_revert::UnwrapOrRevert,
};
pub use casper_types;
use casper_types::{URef, U512};
use odra::types::{Address as OdraAddress, CLValue, EventData, ExecutionError, RuntimeArgs};
pub use odra_casper_shared::{casper_address::CasperAddress, consts};

use crate::casper_env;

static mut ATTACHED_VALUE: U512 = U512::zero();

/// Returns blocktime.
#[no_mangle]
pub fn __get_block_time() -> u64 {
    casper_env::get_block_time()
}

/// Returns contract caller.
#[no_mangle]
pub fn __caller() -> OdraAddress {
    OdraAddress::try_from(casper_env::caller()).unwrap()
}

/// Returns current contract address.
#[no_mangle]
pub fn __self_address() -> OdraAddress {
    OdraAddress::try_from(casper_env::self_address()).unwrap()
}

/// Store a value into the storage.
#[no_mangle]
pub fn __set_var(key: &str, value: &CLValue) {
    casper_env::set_cl_value(key, value.clone());
}

/// Read value from the storage.
#[no_mangle]
pub fn __get_var(key: &str) -> Option<CLValue> {
    casper_env::get_cl_value(key)
}

/// Store the mapping value under a given key.
#[no_mangle]
pub fn __set_dict_value(dict: &str, key: &[u8], value: &CLValue) {
    casper_env::set_dict_value(dict, key, value);
}

/// Read value from the mapping.
#[no_mangle]
pub fn __get_dict_value(dict: &str, key: &[u8]) -> Option<CLValue> {
    casper_env::get_dict_value(dict, key)
}

/// Revert the execution.
#[no_mangle]
pub fn __revert(reason: &ExecutionError) -> ! {
    casper_env::revert(reason.code());
}

// #[no_mangle]
// fn __print(message: &str) {
//     casper_env::print(message);
// }

/// Call another contract.
#[no_mangle]
pub fn __call_contract(
    address: &OdraAddress,
    entrypoint: &str,
    args: &RuntimeArgs,
    amount: Option<U512>,
) -> Vec<u8> {
    let casper_address = CasperAddress::try_from(*address).unwrap();

    if let Some(amount) = amount {
        casper_env::call_contract_with_amount(casper_address, entrypoint, args.clone(), amount)
    } else {
        casper_env::call_contract(casper_address, entrypoint, args.clone())
    }
}

/// Emit event.
#[no_mangle]
pub fn __emit_event(event: &EventData) {
    casper_env::emit_event(event);
}

/// Returns the value that represents one CSPR.
///
/// 1 CSPR = 1,000,000,000 Motes.
#[no_mangle]
pub fn __one_token() -> U512 {
    U512::from(1_000_000_000)
}

/// Returns the balance of the account associated with the currently executing contract.
#[no_mangle]
pub fn __self_balance() -> U512 {
    casper_env::self_balance()
}

/// Returns amount of native token attached to the call.
#[no_mangle]
pub fn __attached_value() -> U512 {
    unsafe { ATTACHED_VALUE }
}

/// Transfers native token from the contract caller to the given address.
#[no_mangle]
pub fn __transfer_tokens(to: OdraAddress, amount: U512) {
    let casper_address = CasperAddress::try_from(to).unwrap();
    let main_purse = get_main_purse();

    match casper_address {
        CasperAddress::Account(account) => {
            transfer_from_purse_to_account(main_purse, account, amount, None).unwrap_or_revert();
        }
        CasperAddress::Contract(_) => __revert(&ExecutionError::can_not_transfer_to_contract()),
    };
}

/// Checks if given named argument exists.
pub fn named_arg_exists(name: &str) -> bool {
    let mut arg_size: usize = 0;
    let ret = unsafe {
        casper_contract::ext_ffi::casper_get_named_arg_size(
            name.as_bytes().as_ptr(),
            name.len(),
            &mut arg_size as *mut usize,
        )
    };
    casper_types::api_error::result_from(ret).is_ok()
}

/// Gets the currently executing contract main purse [URef].
pub fn get_main_purse() -> URef {
    casper_env::get_or_create_purse()
}

/// Stores in memory the amount attached to the current call.
pub fn set_attached_value(amount: U512) {
    unsafe {
        ATTACHED_VALUE = amount;
    }
}

/// Zeroes the amount attached to the current call.
pub fn clear_attached_value() {
    unsafe { ATTACHED_VALUE = U512::zero() }
}

/// Transfers attached value to the currently executing contract.
pub fn handle_attached_value() {
    if named_arg_exists(consts::CARGO_PURSE_ARG) {
        let cargo_purse =
            casper_contract::contract_api::runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = casper_contract::contract_api::system::get_purse_balance(cargo_purse);
        if let Some(amount) = amount {
            let contract_purse = get_main_purse();
            let _ = casper_contract::contract_api::system::transfer_from_purse_to_purse(
                cargo_purse,
                contract_purse,
                amount,
                None,
            );
            set_attached_value(amount);
        }
    }
}

/// Reverts with an [ExecutionError] if some value is attached to the call.
pub fn assert_no_attached_value() {
    if named_arg_exists(consts::CARGO_PURSE_ARG) {
        let cargo_purse =
            casper_contract::contract_api::runtime::get_named_arg(consts::CARGO_PURSE_ARG);
        let amount = casper_contract::contract_api::system::get_purse_balance(cargo_purse);
        if amount.is_some() && !amount.unwrap().is_zero() {
            __revert(&ExecutionError::non_payable());
        }
    }
}
