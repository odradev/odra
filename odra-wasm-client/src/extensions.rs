use casper_types::{AsymmetricType, Transaction};
use js_sys::Promise;
use serde_json::{json, Value};
use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen_futures::JsFuture;

pub trait TransactionExt: Sized {
    fn to_json_string(&self) -> Result<String, JsError>;
    fn add_signature(&self, public_key: &str, signature: &str) -> Result<Self, String>;
}

impl TransactionExt for Transaction {
    fn to_json_string(&self) -> Result<String, JsError> {
        serde_json::to_string(&self).map_err(|e| JsError::new(&e.to_string()))
    }

    fn add_signature(&self, public_key: &str, signature: &str) -> Result<Self, String> {
        // Serialize the existing approvals to JSON
        let casper_transaction = self.clone();
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
}

pub(crate) trait PromiseExt {
    async fn into_js_value(self, context: &str) -> Result<JsValue, JsError>;
}

impl PromiseExt for Result<Promise, JsValue> {
    async fn into_js_value(self, context: &str) -> Result<JsValue, JsError> {
        JsFuture::from(self.with_js_context(context)?)
            .await
            .with_js_context(context)
    }
}

// Trait for converting errors to JsError with context
pub(crate) trait JsErrorContext<T> {
    fn with_js_context(self, context: &str) -> Result<T, JsError>;
}

impl<T, E: std::fmt::Debug> JsErrorContext<T> for Result<T, E> {
    fn with_js_context(self, context: &str) -> Result<T, JsError> {
        self.map_err(|err| JsError::new(&format!("{}: {err:?}", context)))
    }
}
