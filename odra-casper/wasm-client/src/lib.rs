mod casper_wallet;
mod client;
mod imports;
mod odra_client;
mod schemas;
mod utils;
mod wasm;
mod deploy_util;

pub use wasm::deploy_wasm;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
