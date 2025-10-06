use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value;
use wasm_bindgen::{JsError, JsValue};

use crate::cspr_click::{
    callbacks::{ACCOUNT, CALLBACKS},
    types::WrappedAccountInfo
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    SignedIn,
    Disconnected,
    SwitchAccount,
    UnsolicitedAccountChange,
    SignedOut,
    Loaded
}

impl Event {
    pub fn as_str(&self) -> &'static str {
        match self {
            Event::SignedIn => "csprclick:signed_in",
            Event::Disconnected => "csprclick:disconnected",
            Event::SwitchAccount => "csprclick:switched_account",
            Event::UnsolicitedAccountChange => "csprclick:unsolicited_account_change",
            Event::SignedOut => "csprclick:signed_out",
            Event::Loaded => "csprclick:loaded"
        }
    }

    pub fn closure(self) -> wasm_bindgen::closure::Closure<dyn Fn(JsValue)> {
        wasm_bindgen::closure::Closure::<dyn Fn(JsValue)>::new(move |evt: JsValue| {
            self.log(&evt);
            let account = evt.into_serde::<WrappedAccountInfo>();
            if let Ok(account) = account {
                CALLBACKS.with(|callbacks| {
                    if let Some(ref cb) = callbacks.borrow().events.get(&self) {
                        let _ = cb.call1(&JsValue::NULL, &JsValue::from(account.account));
                        ACCOUNT.with(|account| {
                            *account.borrow_mut() = evt;
                        });
                    } else {
                        crate::js::log(&format!("No callback registered for event: {:?}", self));
                    }
                });
            } else {
                if self != Event::SignedOut {
                    ACCOUNT.with(|account| {
                        *account.borrow_mut() = JsValue::NULL;
                    });
                }
                CALLBACKS.with(|callbacks| {
                    if let Some(ref cb) = callbacks.borrow().events.get(&self) {
                        let _ = cb.call0(&JsValue::NULL);
                    } else {
                        crate::js::log(&format!("No callback registered for event: {:?}", self));
                    }
                });
                crate::js::log(&format!("Failed to parse account info: {evt:?}"));
            }
        })
    }

    pub fn log(&self, data: &JsValue) {
        if let Ok(string) = self.to_json_string(data) {
            crate::js::log(&format!("Event triggered: {}: {}", self.as_str(), string));
        } else {
            crate::js::log(&format!(
                "Event triggered: {}: [Failed to serialize data]",
                self.as_str()
            ));
        }
    }

    fn to_json_string(&self, data: &JsValue) -> Result<String, JsError> {
        let serde_value: Value = data.into_serde()?;
        serde_json::to_string(&serde_value)
            .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
    }
}
