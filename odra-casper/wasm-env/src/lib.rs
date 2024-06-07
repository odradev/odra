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

pub use casper_contract;
pub use wasm_contract_env::WasmContractEnv;

// #[cfg(all(target_arch = "wasm32", not(feature = "disable-allocator")))]
// #[allow(unused_imports)]
// use ink_allocator;

// /// Panic handler for the WASM target architecture.
// #[cfg(target_arch = "wasm32")]
// #[panic_handler]
// #[no_mangle]
// pub fn panic(_info: &core::panic::PanicInfo) -> ! {
//     core::intrinsics::abort();
// }
