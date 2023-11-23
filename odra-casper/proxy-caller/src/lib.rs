#![no_std]
#![cfg_attr(not(test), feature(core_intrinsics))]

extern crate alloc;

use core::mem::MaybeUninit;

use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{
        self, account,
        runtime::{self, get_named_arg, revert},
        storage, system
    },
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert
};
use casper_types::{
    api_error,
    bytesrepr::{Bytes, FromBytes, ToBytes},
    ApiError, CLTyped, ContractPackageHash, ContractVersion, RuntimeArgs, URef, U512
};
use odra::consts::{
    ARGS_ARG, ATTACHED_VALUE_ARG, CARGO_PURSE_ARG, CARGO_PURSE_KEY, CONTRACT_PACKAGE_HASH_ARG,
    ENTRY_POINT_ARG
};

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports, clippy::single_component_path_imports)]
use ink_allocator;

/// Contract call definition.
pub struct ProxyCall {
    pub contract_package_hash: ContractPackageHash,
    pub entry_point_name: String,
    pub runtime_args: RuntimeArgs,
    pub attached_value: U512
}

impl ProxyCall {
    /// Load proxy call arguments from the runtime.
    pub fn load_from_args() -> ProxyCall {
        let contract_package_hash = get_named_arg(CONTRACT_PACKAGE_HASH_ARG);
        let entry_point_name = get_named_arg(ENTRY_POINT_ARG);
        let runtime_args: Bytes = get_named_arg(ARGS_ARG);
        let (mut runtime_args, bytes) = RuntimeArgs::from_bytes(&runtime_args).unwrap_or_revert();
        if !bytes.is_empty() {
            revert(ApiError::Deserialize);
        };
        let attached_value: U512 = get_named_arg(ATTACHED_VALUE_ARG);

        if attached_value > U512::zero() {
            let cargo_purse = get_cargo_purse();
            top_up_cargo_purse(cargo_purse, attached_value);
            runtime_args
                .insert(CARGO_PURSE_ARG, cargo_purse)
                .unwrap_or_revert();
        }

        ProxyCall {
            contract_package_hash,
            entry_point_name,
            runtime_args,
            attached_value
        }
    }
}

/// Save value to the storage.
pub fn set_key<T: ToBytes + CLTyped>(name: &str, value: T) {
    match runtime::get_key(name) {
        Some(key) => {
            let key_ref = key.try_into().unwrap_or_revert();
            storage::write(key_ref, value);
        }
        None => {
            let key = storage::new_uref(value).into();
            runtime::put_key(name, key);
        }
    }
}

/// Customized version of `call_versioned_contract` from `casper_contract::contract_api::runtime`.
/// It returns raw bytes instead of deserialized value.
pub fn call_versioned_contract_ret_bytes(
    contract_package_hash: ContractPackageHash,
    entry_point_name: &str,
    runtime_args: RuntimeArgs
) -> Vec<u8> {
    let (contract_package_hash_ptr, contract_package_hash_size, _bytes) =
        to_ptr(contract_package_hash);
    let (contract_version_ptr, contract_version_size, _bytes) = to_ptr(None::<ContractVersion>);
    let (entry_point_name_ptr, entry_point_name_size, _bytes) = to_ptr(entry_point_name);
    let (runtime_args_ptr, runtime_args_size, _bytes) = to_ptr(runtime_args);

    let bytes_written = {
        let mut bytes_written = MaybeUninit::uninit();
        let ret = unsafe {
            ext_ffi::casper_call_versioned_contract(
                contract_package_hash_ptr,
                contract_package_hash_size,
                contract_version_ptr,
                contract_version_size,
                entry_point_name_ptr,
                entry_point_name_size,
                runtime_args_ptr,
                runtime_args_size,
                bytes_written.as_mut_ptr()
            )
        };
        api_error::result_from(ret).unwrap_or_revert();
        unsafe { bytes_written.assume_init() }
    };
    deserialize_contract_result(bytes_written)
}

/// Load or create cargo purse.
fn get_cargo_purse() -> URef {
    match runtime::get_key(CARGO_PURSE_KEY) {
        Some(purse) => purse.into_uref().unwrap_or_revert(),
        None => {
            let purse = system::create_purse();
            runtime::put_key(CARGO_PURSE_KEY, purse.into());
            purse
        }
    }
}

/// Top up cargo purse with the given amount.
/// It reverts if the purse is not empty.
fn top_up_cargo_purse(cargo_purse: URef, amount: U512) {
    let balance = system::get_purse_balance(cargo_purse).unwrap_or_revert();
    if !balance.is_zero() {
        revert(ApiError::Unhandled);
    }

    let main_purse = account::get_main_purse();
    system::transfer_from_purse_to_purse(main_purse, cargo_purse, amount, None).unwrap_or_revert();
}

fn deserialize_contract_result(bytes_written: usize) -> Vec<u8> {
    if bytes_written == 0 {
        // If no bytes were written, the host buffer hasn't been set and hence shouldn't be read.
        Vec::new()
    } else {
        // NOTE: this is a copy of the contents of `read_host_buffer()`.  Calling that directly from
        // here causes several contracts to fail with a Wasmi `Unreachable` error.
        let bytes_non_null_ptr = contract_api::alloc_bytes(bytes_written);
        let mut dest: Vec<u8> = unsafe {
            Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), bytes_written, bytes_written)
        };
        read_host_buffer_into(&mut dest).unwrap_or_revert();
        dest
    }
}

fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
}

fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}
