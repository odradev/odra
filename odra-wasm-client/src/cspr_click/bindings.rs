use super::js;
use crate::{
    cspr_click::{
        callbacks::CALLBACKS,
        types::{AccountInfo, SignResult, TransactionStatus, WrappedAccountInfo},
        TransactionResult
    },
    extensions::{IntoJsError, PromiseExt, TransactionExt},
    types::{Address, PublicKey}
};
use casper_types::Transaction;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Promise;
use serde_json::json;
use wasm_bindgen::prelude::*;

pub(crate) struct CsprClick;

impl CsprClick {
    pub async fn sign_in() -> Result<(), JsError> {
        js::sign_in().into_js_error("[signIn]")
    }

    pub async fn sign_out() -> Result<(), JsError> {
        js::sign_out().into_js_error("[signOut]")
    }

    pub async fn connect(provider: &str) -> Result<AccountInfo, JsError> {
        let result = js::connect(provider)
            .into_js_value("[connect]")
            .await?
            .into_serde::<WrappedAccountInfo>()?;
        Ok(result.account)
    }

    pub async fn disconnect() -> Result<bool, JsError> {
        js::disconnect()
            .into_js_value("[disconnect]")
            .await?
            .as_bool()
            .ok_or_else(|| JsError::new("disconnect failed"))
    }

    #[allow(unused)]
    pub async fn sign_transaction(transaction: Transaction) -> Result<Transaction, JsError> {
        let public_key = Self::get_active_public_key().await?;

        let transaction_json = transaction
            .to_json_string()
            .into_js_error("Failed to serialize transaction")?;

        let result =
            Self::process_sign_result(js::sign_transaction(&transaction_json, &public_key)).await?;
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
        let public_key = js::get_active_public_key()
            .into_js_value("[getActivePublicKey]")
            .await?
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

        let result = js::send(
            &transaction_json,
            &public_key,
            on_status_update.as_ref().unchecked_ref()
        )
        .into_js_value("[send]")
        .await?
        .into_serde::<TransactionResult>()?;
        on_status_update.forget(); // Prevent the closure from being dropped
        Ok(result)
    }

    pub async fn get_active_account() -> Result<AccountInfo, JsError> {
        let json = json!({ "withBalance": true });
        let options = JsValue::from_serde(&json).into_js_error("Failed to serialize options")?;
        let result = js::get_active_account(&options)
            .into_js_value("[getActiveAccount]")
            .await?
            .into_serde::<WrappedAccountInfo>()?;
        Ok(result.account)
    }

    pub async fn is_unlocked(provider: &str) -> Result<bool, JsError> {
        js::is_unlocked(provider)
            .into_js_value("[isUnlocked]")
            .await?
            .as_bool()
            .ok_or_else(|| JsError::new("isUnlocked failed"))
    }

    pub async fn sign_in_with_account(account: AccountInfo) -> Result<AccountInfo, JsError> {
        let result = js::sign_in_with_account(account)
            .into_js_value("[signInWithAccount]")
            .await?
            .into_serde()?;
        Ok(result)
    }

    pub async fn switch_account() -> Result<(), JsError> {
        js::switch_account().into_js_value("[switchAccount").await?;
        Ok(())
    }

    pub async fn sign_message(message: &str) -> Result<SignResult, JsError> {
        let public_key = Self::get_active_public_key().await?;
        Self::process_sign_result(js::sign_message(message, &public_key)).await
    }

    async fn process_sign_result(promise: Result<Promise, JsValue>) -> Result<SignResult, JsError> {
        let result: SignResult = promise.into_js_error("[signMessage]")?.into_serde()?;
        Ok(result)
    }
}
