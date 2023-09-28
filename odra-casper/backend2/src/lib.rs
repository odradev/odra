#![no_std]
#![cfg_attr(not(test), feature(core_intrinsics))]

extern crate alloc;

pub mod wasm_contract_env;
// pub mod types;
// pub mod env;
// pub mod variable;
pub mod consts;
pub mod wasm_host;
// pub mod module;
// pub mod mapping;
pub use casper_contract;

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use ink_allocator;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}
