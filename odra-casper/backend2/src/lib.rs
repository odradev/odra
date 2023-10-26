#![no_std]
#![cfg_attr(not(test), feature(core_intrinsics))]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

pub mod casper_contract_env;
pub mod consts;
pub mod wasm_host;

pub use casper_contract;
pub use casper_contract_env::WasmContractEnv;

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use ink_allocator;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}
