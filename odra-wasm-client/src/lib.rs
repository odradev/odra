use wasm_bindgen::prelude::*;

mod client;
mod contracts;
pub mod cspr_click;
mod examples;
mod extensions;
pub mod js;
pub mod types;
mod utils;

pub use client::OdraWasmClient;

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
