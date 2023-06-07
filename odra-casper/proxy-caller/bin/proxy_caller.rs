#![no_std]
#![no_main]

extern crate alloc;

use casper_contract::contract_api::runtime::call_versioned_contract;
use odra_casper_proxy_caller::{attach_cspr, load_args, ProxyCall};

#[no_mangle]
fn call() {
    let mut proxy_call: ProxyCall = load_args();
    attach_cspr(&mut proxy_call);
    let _: () = call_versioned_contract(
        proxy_call.contract_package_hash,
        None,
        proxy_call.entry_point_name.as_str(),
        proxy_call.runtime_args
    );
}
