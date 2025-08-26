use casper_types::Timestamp;
use js_sys::Date;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn default_payment() -> u64 {
    2_500_000_000
}

// export macro
// pub use odra_wasm_client_macro::wasm_client;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
