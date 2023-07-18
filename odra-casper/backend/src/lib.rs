#![no_std]
//! Generate Casper contract and interact with Casper host.
#![cfg_attr(not(test), feature(core_intrinsics))]

extern crate alloc;

mod casper_env;
pub mod contract_env;
pub mod utils;

pub use casper_contract::{
    self,
    contract_api::{runtime, storage}
};

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
use ink_allocator;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}