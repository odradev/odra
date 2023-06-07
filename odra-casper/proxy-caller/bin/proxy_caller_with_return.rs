#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use casper_types::bytesrepr::Bytes;
use odra_casper_proxy_caller::{
    attach_cspr, call_versioned_contract_ret_bytes, load_args, set_key, ProxyCall
};
use odra_casper_shared::consts::RESULT_KEY;

#[no_mangle]
fn call() {
    let mut proxy_call: ProxyCall = load_args();
    attach_cspr(&mut proxy_call);
    let result: Vec<u8> = call_versioned_contract_ret_bytes(
        proxy_call.contract_package_hash,
        proxy_call.entry_point_name.as_str(),
        proxy_call.runtime_args
    );
    set_key(RESULT_KEY, Bytes::from(result));
}
