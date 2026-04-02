#![doc = "WASM environment for Odra Framework"]
#![doc = "It is an implementation of the contract environment used by the contracts written in Odra,"]
#![doc = "which are compiled to the WASM target architecture."]
#![no_std]
#![allow(internal_features)]
#![cfg_attr(not(test), feature(core_intrinsics))]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

pub(crate) mod consts;
pub mod host_functions;
mod wasm_contract_env;

pub use crate::wasm_contract_env::WasmContractEnv;
use alloc::rc::Rc;
pub use casper_contract;

#[cfg(all(target_arch = "wasm32", not(feature = "disable-allocator")))]
#[allow(unused_imports)]
use ink_allocator;
use odra_core::casper_event_standard::Schemas;
use odra_core::prelude::{ExecutionError, Revertible};
use odra_core::ExecutionEnv;

/// Panic handler for the WASM target architecture.
#[cfg(target_arch = "wasm32")]
#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}

/// This function is used to migrate the contract's events schemas during the upgrade process.
#[no_mangle]
pub fn migrate_events() {
    let exec_env = {
        let env = WasmContractEnv::new_env();
        let env_rc = Rc::new(env);
        ExecutionEnv::new(env_rc)
    };
    let schemas: Schemas = exec_env.get_named_arg("schemas");

    exec_env.migrate_schemas(schemas.0);
}
