#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary only supports wasm32 target architecture!");

extern crate alloc;

use odra_casper_proxy_caller::{call_versioned_contract_ret_bytes, set_key, ProxyCall};
use odra_core::casper_types::bytesrepr::Bytes;
use odra_core::consts::RESULT_KEY;
use odra_core::prelude::*;

#[no_mangle]
fn call() {
    let proxy_call = ProxyCall::load_from_args();
    let result: Vec<u8> = call_versioned_contract_ret_bytes(
        proxy_call.contract_package_hash,
        proxy_call.entry_point_name.as_str(),
        proxy_call.runtime_args
    );
    set_key(RESULT_KEY, Bytes::from(result));
}
