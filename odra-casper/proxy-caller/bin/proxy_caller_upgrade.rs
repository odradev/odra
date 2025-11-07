#![doc = "Proxy Caller with return binary - to be compiled into the WASM"]
#![no_std]
#![no_main]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary only supports wasm32 target architecture!");

extern crate alloc;

use odra_casper_wasm_env::casper_contract::contract_api::runtime;
use odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert;
use odra_core::consts::{ARGS_ARG, PACKAGE_HASH_ARG};
use odra_core::casper_types::bytesrepr::FromBytes;

#[no_mangle]
fn call() {
    let package_hash = runtime::get_named_arg(PACKAGE_HASH_ARG);
    let package_name: odra_core::prelude::string::String = runtime::get_named_arg("package_name");
    let runtime_args: odra_core::casper_types::bytesrepr::Bytes = runtime::get_named_arg(ARGS_ARG);
    let (runtime_args, _bytes) = odra_core::casper_types::RuntimeArgs::from_bytes(&runtime_args).unwrap_or_revert();
    runtime::print("Proxy caller - upgrade_child_contract");

    let (_, access_uref): (odra_core::casper_types::contracts::ContractPackageHash, odra_core::casper_types::URef) = runtime::call_versioned_contract(
        package_hash,
        None,
        "upgrade_child_contract",
        runtime_args
    );
    runtime::put_key(&package_name, odra_core::casper_types::Key::URef(access_uref));
}
