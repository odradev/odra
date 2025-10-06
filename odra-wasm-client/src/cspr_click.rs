use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{
    cspr_click::{
        callbacks::{ACCOUNT, CALLBACKS},
        event::Event,
        types::{AccountType, SignResult, TransactionStatus, WrappedAccountType}
    },
    types::{Address, PublicKey, Transaction}
};

pub(crate) mod callbacks;
mod event;
pub(crate) mod js;
mod types;

pub use types::TransactionResult;

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
        js::sign_in().map_err(|err| JsError::new(&format!("signIn failed: {err:?}")))
    }

    pub async fn disconnect() -> Result<bool, JsError> {
        let disconnected = JsFuture::from(
            js::disconnect().map_err(|err| JsError::new(&format!("disconnect failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("disconnect failed: {err:?}")))?;

        disconnected
            .as_bool()
            .ok_or_else(|| JsError::new("disconnect failed"))
    }

    pub async fn sign_transaction(transaction: Transaction) -> Result<Transaction, JsError> {
        let public_key = Self::get_active_public_key().await?;

        let transaction_json = transaction
            .to_json_string()
            .map_err(|err| JsError::new(&format!("Failed to serialize transaction: {err:?}")))?;

        let sign = JsFuture::from(
            js::sign(&transaction_json, &public_key.to_string())
                .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Signing failed: {err:?}")))?;

        let result: SignResult = sign
            .into_serde()
            .map_err(|err| JsError::new(&format!("Deserialize signature failed: {err:?}")))?;

        if result.is_cancelled {
            return Err(JsError::new(&format!(
                "Could not sign transaction for key {public_key}"
            )));
        }
        let signature = String::from_utf8(result.signature).unwrap_or_default();
        transaction
            .add_signature(&public_key.to_string(), &signature)
            .map_err(|e| JsError::new(&format!("Failed to add signature to transaction: {e:?}")))
    }

    pub async fn get_active_public_key() -> Result<String, JsError> {
        let public_key = JsFuture::from(
            js::get_active_public_key()
                .map_err(|err| JsError::new(&format!("getActivePublicKey failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("getActivePublicKey failed: {err:?}")))?;

        let public_key = public_key
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
            .map_err(|err| JsError::new(&format!("Failed to serialize transaction: {err:?}")))?;

        let sent = JsFuture::from(
            js::send(
                &transaction_json,
                &public_key,
                on_status_update.as_ref().unchecked_ref()
            )
            .map_err(|err| JsError::new(&format!("Sending failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("Sending failed: {err:?}")))?;
        let result = sent.into_serde::<TransactionResult>().map_err(|err| {
            JsError::new(&format!(
                "Failed to deserialize send transaction result: {err:?}"
            ))
        })?;
        on_status_update.forget(); // Prevent the closure from being dropped
        Ok(result)
    }

    pub async fn get_active_account() -> Result<AccountType, JsError> {
        JsFuture::from(
            js::get_active_account()
                .map_err(|err| JsError::new(&format!("getActiveAccount failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("getActiveAccount failed: {err:?}")))?
        .into_serde()
        .map_err(|err| JsError::new(&format!("Deserialize account failed: {err:?}")))
    }

    pub async fn is_unlocked(provider: &str) -> Result<bool, JsError> {
        let unlocked = JsFuture::from(
            js::is_unlocked(provider)
                .map_err(|err| JsError::new(&format!("isUnlocked failed: {err:?}")))?
        )
        .await
        .map_err(|err| JsError::new(&format!("isUnlocked failed: {err:?}")))?;

        unlocked
            .as_bool()
            .ok_or_else(|| JsError::new("isUnlocked failed"))
    }
}
