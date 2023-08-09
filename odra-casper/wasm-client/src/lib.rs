mod schemas;
mod wasm;
mod utils;
mod imports;
mod casper_wallet;
mod client;

pub use wasm::deploy_wasm;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
