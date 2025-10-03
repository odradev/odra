use core::fmt;

use casper_types::{
    AsymmetricType, Deploy, Digest as _Digest, Transaction as _Transaction,
    TransactionHash as _TransactionHash, TransactionV1
};
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize, Clone, Serialize)]
#[wasm_bindgen]
pub struct Transaction(_Transaction);

#[wasm_bindgen]
impl Transaction {
    #[wasm_bindgen(constructor)]
    pub fn new(transaction: JsValue) -> Self {
        let transaction: _Transaction = transaction.into_serde().unwrap();
        transaction.into()
    }

    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json_js_alias(&self) -> JsValue {
        match JsValue::from_serde(&self.0) {
            Ok(json) => json,
            Err(_err) => JsValue::null()
        }
    }
}

impl Transaction {
    pub fn add_signature(&self, public_key: &str, signature: &str) -> Result<Self, String> {
        // Serialize the existing approvals to JSON
        let casper_transaction: _Transaction = self.0.clone();
        let existing_approvals_json = casper_transaction
            .approvals()
            .iter()
            .map(|approval| {
                json!({
                    "signer": approval.signer().to_hex(),
                    "signature": approval.signature().to_hex(),
                })
            })
            .collect::<Vec<_>>();

        // Create JSON object for the new approval
        let new_approval_json = json!({
            "signer": public_key,
            "signature": signature,
        });

        // Append the new approval to existing approvals
        let mut all_approvals_json = existing_approvals_json;
        all_approvals_json.push(new_approval_json);

        // Convert the approvals JSON back to string
        let updated_approvals_str = serde_json::to_string(&all_approvals_json)
            .map_err(|_| "Failed to serialize updated approvals JSON")?;

        // Replace the approvals field in the original transaction JSON string
        let mut transaction_json: Value = serde_json::from_str(&self.to_json_string().unwrap())
            .map_err(|_| "Failed to deserialize transaction JSON")?;
        transaction_json["Version1"]["approvals"] = serde_json::from_str(&updated_approvals_str)
            .map_err(|_| "Failed to deserialize updated approvals JSON")?;

        // Convert the updated transaction JSON back to a Transaction struct
        let updated_transaction: Transaction = serde_json::from_value(transaction_json)
            .map_err(|_| "Failed to deserialize updated transaction JSON")?;

        Ok(updated_transaction)
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string(&self.0).map_err(|e| e.to_string())
    }
}

impl From<Transaction> for _Transaction {
    fn from(transaction: Transaction) -> Self {
        transaction.0
    }
}

impl From<_Transaction> for Transaction {
    fn from(transaction: _Transaction) -> Self {
        Transaction(transaction)
    }
}

impl From<Deploy> for Transaction {
    fn from(deploy: Deploy) -> Self {
        _Transaction::Deploy(deploy).into()
    }
}

impl From<TransactionV1> for Transaction {
    fn from(transaction: TransactionV1) -> Self {
        _Transaction::V1(transaction).into()
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[wasm_bindgen]
pub struct TransactionHash(_TransactionHash);

impl TransactionHash {
    pub fn new(transaction_hash_hex_str: &str) -> Result<Self, String> {
        let bytes = hex::decode(transaction_hash_hex_str)
            .map_err(|err| format!("Failed to decode hex: {err}"))?;

        Self::from_raw(&bytes)
    }

    pub fn from_raw(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != _Digest::LENGTH {
            return Err(format!("Incorrect digest length: {}", bytes.len()));
        }

        let mut hash = [0u8; _Digest::LENGTH];
        hash.copy_from_slice(bytes);
        Ok(Self(_TransactionHash::from_raw(hash)))
    }
}

#[wasm_bindgen]
impl TransactionHash {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn new_js_alias(transaction_hash_hex_str: &str) -> Result<TransactionHash, JsError> {
        TransactionHash::new(transaction_hash_hex_str)
            .map_err(|err| JsError::new(&format!("{err:?}")))
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = "fromRaw")]
    pub fn from_raw_js_alias(bytes: &[u8]) -> Result<TransactionHash, JsError> {
        TransactionHash::from_raw(bytes).map_err(|err| JsError::new(&format!("{err:?}")))
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = "toJson")]
    pub fn to_json(&self) -> JsValue {
        JsValue::from_serde(self).unwrap_or(JsValue::null())
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string_js_alias(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<TransactionHash> for _TransactionHash {
    fn from(transaction_hash: TransactionHash) -> Self {
        transaction_hash.0
    }
}

impl From<&TransactionHash> for _TransactionHash {
    fn from(transaction_hash: &TransactionHash) -> Self {
        transaction_hash.0
    }
}

impl From<_TransactionHash> for TransactionHash {
    fn from(transaction_hash: _TransactionHash) -> Self {
        TransactionHash(transaction_hash)
    }
}
