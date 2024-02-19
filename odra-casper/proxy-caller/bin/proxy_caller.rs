#![doc = "Proxy Caller binary - to be compiled into the WASM"]
#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary only supports wasm32 target architecture!");

extern crate alloc;

use odra_casper_proxy_caller::ProxyCall;
use odra_casper_wasm_env::casper_contract::contract_api::runtime::call_versioned_contract;

#[no_mangle]
fn call() {
    let proxy_call = ProxyCall::load_from_args();
    let _: () = call_versioned_contract(
        proxy_call.contract_package_hash,
        None,
        proxy_call.entry_point_name.as_str(),
        proxy_call.runtime_args
    );
}
