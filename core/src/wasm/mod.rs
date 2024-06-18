//! Wasm environment.

pub mod consts;
pub mod host_functions;
pub mod wasm_contract_env;
pub use casper_contract;

pub use wasm_contract_env::WasmContractEnv;

#[cfg(all(target_arch = "wasm32", not(feature = "disable-allocator")))]
#[allow(unused_imports)]
use ink_allocator;

/// Panic handler for the WASM target architecture.
#[cfg(target_arch = "wasm32")]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort();
}
