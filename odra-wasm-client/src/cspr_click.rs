use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;

use crate::cspr_click::{callbacks::ACCOUNT, event::Event, types::WrappedAccountInfo};

mod bindings;
pub(crate) mod callbacks;
mod event;
pub(crate) mod js;
mod types;

pub(crate) use bindings::CsprClick;
pub use types::{AccountInfo, SignResult, TransactionResult};

macro_rules! register_cspr_event {
    ($ev:expr, $closure:ident) => {
        let _ = js::on_csprclick_event($ev.as_str(), $closure.as_ref().unchecked_ref());
    };
}

#[wasm_bindgen(js_name = "getCurrentAccount")]
pub fn get_account() -> Result<AccountInfo, JsError> {
    ACCOUNT
        .with(|account| account.borrow().clone().into_serde::<WrappedAccountInfo>())
        .map(|wrapped| wrapped.account)
        .map_err(|e| JsError::new(&format!("Failed to parse account: {}", e)))
}

pub(crate) fn init() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let on_signed_in = Event::SignedIn.closure();
    let on_switched_account = Event::SwitchAccount.closure();
    let on_unsolicited_account_change = Event::UnsolicitedAccountChange.closure();
    let on_signed_out = Event::SignedOut.closure();
    let on_disconnected = Event::Disconnected.closure();
    let on_loaded = Closure::<dyn Fn()>::new(move || {
        register_cspr_event!(Event::SignedIn, on_signed_in);
        register_cspr_event!(Event::SwitchAccount, on_switched_account);
        register_cspr_event!(
            Event::UnsolicitedAccountChange,
            on_unsolicited_account_change
        );
        register_cspr_event!(Event::SignedOut, on_signed_out);
        register_cspr_event!(Event::Disconnected, on_disconnected);
    });
    window.add_event_listener_with_callback(
        Event::Loaded.as_str(),
        on_loaded.as_ref().unchecked_ref()
    )?;
    on_loaded.forget();
    Ok(())
}
