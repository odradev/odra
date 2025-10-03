use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::types::Transaction;

const USER_ERR_PREFIX: &str = "User error: ";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignResult {
    #[serde(rename = "cancelled")]
    pub is_cancelled: bool,
    #[serde(rename = "signatureHex")]
    pub signature_hex: Option<String>,
    pub signature: Vec<u8>,
    pub transaction: Option<Transaction>,
    pub error: Option<String>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct AccountType {
    pub provider: String,
    #[serde(rename = "providerSupports")]
    pub provider_supports: Option<Vec<String>>,
    pub cspr_name: Option<String>,
    pub public_key: Option<String>,
    pub connected_at: u64,
    pub token: Option<String>,
    custom: Option<serde_json::Value>,
    pub balance: Option<String>,
    pub liquid_balance: Option<String>,
    pub logo: Option<String>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WrappedAccountType {
    pub account: AccountType
}


#[wasm_bindgen]
pub enum TransactionStatus {
    /// The transaction has been signed and successfully deployed to a Casper node.
    SENT,
    /// The transaction has been processed by the network. May result in success or failure.
    PROCESSED,
    /// The transaction’s time-to-live (TTL) elapsed before execution.
    EXPIRED,
    /// The user rejected the signature request.
    CANCELLED,
    /// The SDK stopped listening for updates before the transaction was finalized. A custom timeout can be specified (default: 120 seconds).
    TIMEOUT,
    /// An unexpected error occurred while submitting or monitoring the transaction.
    ERROR,
    /// A heartbeat event sent periodically to indicate that the connection is still active.
    PING
}

impl TransactionStatus {
    pub fn from_str(status: &str) -> Result<Self, String> {
        match status {
            "sent" => Ok(TransactionStatus::SENT),
            "processed" => Ok(TransactionStatus::PROCESSED),
            "expired" => Ok(TransactionStatus::EXPIRED),
            "cancelled" => Ok(TransactionStatus::CANCELLED),
            "timeout" => Ok(TransactionStatus::TIMEOUT),
            "error" => Ok(TransactionStatus::ERROR),
            "ping" => Ok(TransactionStatus::PING),
            _ => Err(format!("Unknown transaction status: {}", status))
        }
    }
}

// fn jsvalue_to_json_string(value: &JsValue) -> Result<String, Box<dyn std::error::Error>> {
//     let mut  serde_value: Value = value.into_serde()?;
//     serde_value
//     Ok(serde_json::to_string(&serde_value)?)
// }

pub(super) fn add_odra_error_value(js_value: &JsValue) -> Result<JsValue, JsError> {
    let mut value: Value = js_value.into_serde()?;
    let error = value
        .as_object()
        .map(|obj| obj.get("error").map(|e| e.as_str()))
        .flatten()
        .flatten();
    if let Some(error) = error {
        let odra_err = if error == "Out of gas error" {
            Some(odra_core::prelude::ExecutionError::OutOfGas.code())
        } else {
            error
                .strip_prefix(USER_ERR_PREFIX)
                .and_then(|s| s.parse().ok())
        };
        if let Some(code) = odra_err {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("odraErrorCode".to_string(), Value::Number(code.into()));
            }
        }
    }
    JsValue::from_serde(&value)
        .map_err(|e| JsError::new(&format!("Failed to serialize value: {}", e)))
}
