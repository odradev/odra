use wasm_bindgen::prelude::*;

#[cfg(feature = "cep18")]
mod cep18;
mod client;
mod contracts;
mod cspr_click;
pub mod js;
pub mod types;
mod utils;
mod wallet;
#[cfg(feature = "wcspr")]
mod wcspr;

pub use client::OdraWasmClient;
pub use wallet::CasperWallet;

pub const PROXY_CALLER: &[u8; 52814] = include_bytes!("../proxy_caller_with_return.wasm");

pub use gloo_utils::format::JsValueSerdeExt;
pub use odra_core::casper_types;
pub use odra_core::prelude::Address as OdraAddress;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;

#[wasm_bindgen(start)]
fn run() -> Result<(), JsValue> {
    cspr_click::init()?;
    js::log("WASM module loaded");
    Ok(())
}
