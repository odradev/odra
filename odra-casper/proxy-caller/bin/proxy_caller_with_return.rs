#![doc = "Proxy Caller with return binary - to be compiled into the WASM"]
#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary only supports wasm32 target architecture!");

extern crate alloc;

use odra_casper_proxy_caller::{
    call_versioned_contract_ret_bytes, ensure_cargo_purse_is_empty, set_key, ProxyCall
};
use odra_core::casper_types::bytesrepr::Bytes;
use odra_core::consts::RESULT_KEY;
use odra_core::prelude::*;

#[no_mangle]
fn call() {
    let proxy_call = ProxyCall::load_from_args();
    let result: Vec<u8> = call_versioned_contract_ret_bytes(
        proxy_call.package_hash,
        proxy_call.entry_point_name.as_str(),
        proxy_call.runtime_args
    );
    ensure_cargo_purse_is_empty(proxy_call.attached_value);
    set_key(RESULT_KEY, Bytes::from(result));
}
