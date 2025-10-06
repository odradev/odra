use gloo_utils::format::JsValueSerdeExt;
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{
    cspr_click::{
        callbacks::{ACCOUNT, CALLBACKS},
        event::Event,
        types::{SignResult, TransactionStatus, WrappedAccountType}
    },
    types::{Address, PublicKey, Transaction}
};

pub(crate) mod callbacks;
mod event;
pub(crate) mod js;
mod types;

pub use types::{AccountType, TransactionResult};

macro_rules! register_cspr_event {
    ($ev:expr, $closure:ident) => {
        let _ = js::on_csprclick_event($ev.as_str(), $closure.as_ref().unchecked_ref());
    };
}

#[wasm_bindgen]
pub fn get_account() -> Result<AccountType, JsError> {
    ACCOUNT
        .with(|account| account.borrow().clone().into_serde::<WrappedAccountType>())
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

pub(crate) struct CsprClick;

impl CsprClick {
    pub async fn sign_in() -> Result<(), JsError> {
        js::sign_in().into_js_error("signIn failed")
    }

    pub async fn disconnect() -> Result<bool, JsError> {
        JsFuture::from(js::disconnect().into_js_error("disconnect failed")?)
            .await
            .into_js_error("disconnect failed")?
            .as_bool()
            .ok_or_else(|| JsError::new("disconnect failed"))
    }

    pub async fn sign_transaction(transaction: Transaction) -> Result<Transaction, JsError> {
        let public_key = Self::get_active_public_key().await?;

        let transaction_json = transaction
            .to_json_string()
            .into_js_error("Failed to serialize transaction")?;

        let promise =
            js::sign(&transaction_json, &public_key).into_js_error("Failed to sign transaction")?;
        let sign = JsFuture::from(promise)
            .await
            .into_js_error("Signing failed")?;

        let result: SignResult = sign
            .into_serde()
            .into_js_error("Deserialize signature failed")?;

        if result.is_cancelled {
            return Err(JsError::new(&format!(
                "Could not sign transaction for key {public_key}"
            )));
        }
        let signature = String::from_utf8(result.signature).unwrap_or_default();
        transaction
            .add_signature(&public_key.to_string(), &signature)
            .into_js_error("Failed to add signature to transaction")
    }

    pub async fn get_active_public_key() -> Result<String, JsError> {
        let promise = js::get_active_public_key().into_js_error("getActivePublicKey failed")?;
        let public_key = JsFuture::from(promise)
            .await
            .into_js_error("getActivePublicKey failed")?
            .as_string()
            .ok_or_else(|| JsError::new("getActivePublicKey failed"))?;

        if public_key.is_empty() {
            return Err(JsError::new("getActivePublicKey returned an empty string"));
        }
        Ok(public_key)
    }

    pub async fn caller() -> Result<Address, JsError> {
        let public_key = Self::get_active_public_key().await?;
        PublicKey::new(&public_key)
            .map_err(|e| JsError::new(&e.to_string()))
            .map(Into::<Address>::into)
    }

    pub async fn send_transaction(
        transaction: Transaction,
        public_key: String
    ) -> Result<TransactionResult, JsError> {
        let on_status_update =
            Closure::<dyn Fn(JsValue, JsValue)>::new(move |status: JsValue, result: JsValue| {
                let parsed_result = result.into_serde::<TransactionResult>();
                let status = status.into_serde::<TransactionStatus>();
                if let (Ok(status), Ok(parsed_result)) = (status, parsed_result) {
                    CALLBACKS.with(|callbacks| {
                        let _ = callbacks.borrow().transaction.call2(
                            &JsValue::NULL,
                            &JsValue::from(status),
                            &JsValue::from(parsed_result)
                        );
                    });
                } else {
                    crate::js::log(&format!(
                        "Failed to parse transaction status update {result:?}"
                    ));
                }
            });

        let transaction_json = transaction
            .to_json_string()
            .into_js_error("Failed to serialize transaction")?;

        let promise = js::send(
            &transaction_json,
            &public_key,
            on_status_update.as_ref().unchecked_ref()
        )
        .into_js_error("Failed to send transaction")?;
        let result = JsFuture::from(promise)
            .await
            .into_js_error("Sending transaction failed")?
            .into_serde::<TransactionResult>()
            .into_js_error("Deserialize transaction result failed")?;
        on_status_update.forget(); // Prevent the closure from being dropped
        Ok(result)
    }

    pub async fn get_active_account() -> Result<AccountType, JsError> {
        let json = json!({ "withBalance": true });
        let options = JsValue::from_serde(&json).into_js_error("Failed to serialize options")?;
        let promise = js::get_active_account(&options).into_js_error("getActiveAccount failed")?;
        JsFuture::from(promise)
            .await
            .into_js_error("getActiveAccount failed")?
            .into_serde()
            .into_js_error("Deserialize account failed")
    }

    pub async fn is_unlocked(provider: &str) -> Result<bool, JsError> {
        let promise = js::is_unlocked(provider).into_js_error("isUnlocked failed")?;
        JsFuture::from(promise)
            .await
            .into_js_error("isUnlocked failed")?
            .as_bool()
            .ok_or_else(|| JsError::new("isUnlocked failed"))
    }

    pub async fn sign_in_with_account(account: AccountType) -> Result<AccountType, JsError> {
        JsFuture::from(js::sign_in_with_account(account).into_js_error("signInWithAccount failed")?)
            .await
            .into_js_error("signInWithAccount failed")?
            .into_serde()
            .into_js_error("Deserialize account failed")
    }
}

// Trait for converting errors to JsError with context
trait IntoJsError<T> {
    fn into_js_error(self, context: &str) -> Result<T, JsError>;
}

impl<T, E: std::fmt::Debug> IntoJsError<T> for Result<T, E> {
    fn into_js_error(self, context: &str) -> Result<T, JsError> {
        self.map_err(|err| JsError::new(&format!("{}: {err:?}", context)))
    }
}

impl<T> IntoJsError<T> for Option<T> {
    fn into_js_error(self, context: &str) -> Result<T, JsError> {
        self.ok_or_else(|| JsError::new(context))
    }
}
