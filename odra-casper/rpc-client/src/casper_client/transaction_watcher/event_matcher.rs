//! Transaction event matcher.
//! Handles matching transaction hashes in Casper network events.

use crate::error::LivenetError;
use crate::error::LivenetError::ClientError;
use serde_json::Value as JsonValue;

/// Checks if a transaction hash matches a TransactionProcessed event.
pub struct EventMatcher;

impl EventMatcher {
    /// Checks if the event contains the specified transaction hash.
    ///
    /// # Arguments
    ///
    /// * `event_json` - The JSON string of the SSE event
    /// * `expected_hash` - The transaction hash we're looking for
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the event matches, `Ok(false)` otherwise, or an error if parsing fails.
    pub fn matches_transaction_hash(
        event_json: &str,
        expected_hash: &str
    ) -> Result<bool, LivenetError> {
        let event: JsonValue = serde_json::from_str(event_json.trim())
            .map_err(|e| ClientError(format!("Failed to parse event JSON: {}", e)))?;

        let transaction_hash = Self::extract_transaction_hash(&event)?;

        Ok(transaction_hash
            .map(|hash| hash == expected_hash)
            .unwrap_or(false))
    }

    /// Extracts the transaction hash from a Casper event.
    ///
    /// The hash can be in two formats:
    /// - `Version1`: Standard transaction hash
    /// - `Deploy`: Deploy transaction hash
    fn extract_transaction_hash(event: &JsonValue) -> Result<Option<String>, LivenetError> {
        let transaction_processed = match event.get("TransactionProcessed") {
            Some(tp) => tp,
            None => return Ok(None)
        };

        let hash_obj = match transaction_processed.get("transaction_hash") {
            Some(ho) => ho,
            None => return Ok(None)
        };

        let hash_str = hash_obj
            .get("Version1")
            .or_else(|| hash_obj.get("Deploy"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(hash_str)
    }
}
