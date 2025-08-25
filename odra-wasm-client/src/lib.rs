use std::str::FromStr;

use casper_types::Timestamp;
use js_sys::Date;

mod cep18;
mod client;
pub mod js;
mod types;
mod wallet;

pub const PROXY_CALLER: &[u8; 52814] = include_bytes!("../proxy_caller_with_return.wasm");

pub(crate) fn now() -> Option<Timestamp> {
    let now = Date::new_0();
    let now_str = now.to_iso_string().as_string()?;
    let timestamp = Timestamp::from_str(&now_str).ok()?;
    Some(timestamp)
}
