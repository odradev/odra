use std::{cell::RefCell, collections::BTreeMap};
use wasm_bindgen::prelude::*;

use crate::cspr_click::event::Event;

#[derive(Default)]
#[wasm_bindgen]
pub struct CsprClickCallbacks {
    pub(crate) events: BTreeMap<Event, js_sys::Function>,
    pub(crate) transaction: js_sys::Function
}

#[wasm_bindgen]
impl CsprClickCallbacks {
    #[wasm_bindgen(js_name = "onSignedIn")]
    pub fn set_on_signed_in_callback(callback: js_sys::Function) {
        with_callbacks(|callbacks| {
            callbacks.events.insert(Event::SignedIn, callback);
        });
    }

    #[wasm_bindgen(js_name = "onSwitchedAccount")]
    pub fn set_on_switched_account_callback(callback: js_sys::Function) {
        with_callbacks(|callbacks| {
            callbacks.events.insert(Event::SwitchAccount, callback);
        });
    }

    #[wasm_bindgen(js_name = "onSignedOut")]
    pub fn set_on_signed_out_callback(callback: js_sys::Function) {
        with_callbacks(|callbacks| {
            callbacks.events.insert(Event::SignedOut, callback);
        });
    }

    #[wasm_bindgen(js_name = "onTransactionStatusUpdate")]
    pub fn set_on_transaction_update_callback(callback: js_sys::Function) {
        with_callbacks(|callbacks| {
            callbacks.transaction = callback;
        });
    }
}

thread_local! {
    pub static CALLBACKS: RefCell<CsprClickCallbacks> = RefCell::new(CsprClickCallbacks::default());
    pub static ACCOUNT: RefCell<JsValue> = RefCell::new(JsValue::NULL);
}

// Helper function to access callbacks
fn with_callbacks<F, R>(f: F) -> R
where
    F: FnOnce(&mut CsprClickCallbacks) -> R
{
    CALLBACKS.with(|callbacks| f(&mut callbacks.borrow_mut()))
}
