use casper_types::Timestamp;
use js_sys::Date;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[cfg(feature = "example")]
mod cep18;
mod client;
pub mod js;
pub mod types;
mod wallet;

pub use client::OdraWasmClient;
pub use wallet::CasperWallet;

pub const PROXY_CALLER: &[u8; 52814] = include_bytes!("../proxy_caller_with_return.wasm");

pub(crate) fn now() -> Option<Timestamp> {
    let now = Date::new_0();
    let now_str = now.to_iso_string().as_string()?;
    let timestamp = Timestamp::from_str(&now_str).ok()?;
    Some(timestamp)
}

pub use gloo_utils::format::JsValueSerdeExt;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
pub use odra_core::casper_types;
pub use odra_core::prelude::Address as OdraAddress;