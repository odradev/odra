//! Transaction event matcher.
//! Handles matching transaction hashes in Casper network events.

use crate::{error::LivenetError, log};
use casper_types::execution::ExecutionResult;

/// Finds [ExecutionResult] of the transaction with the specified transaction hash.
///
/// # Arguments
///
/// * `event_json` - The JSON string of the SSE event
/// * `expected_hash` - The transaction hash we're looking for
pub fn find_result(
    event_json: &str,
    expected_hash: &str
) -> Result<Option<ExecutionResult>, LivenetError> {
    let event: serde_json::Value = serde_json::from_str(event_json.trim())
        .map_err(|e| LivenetError::ClientError(format!("Failed to parse event JSON: {}", e)))?;

    // The exact struct: https://docs.casper.network/developers/monitor-and-consume-events#transactionprocessed
    match event.get("TransactionProcessed") {
        Some(tx) => {
            if match_hash(tx, expected_hash) {
                let execution_result = tx
                    .get("execution_result")
                    .ok_or(LivenetError::ClientError(
                        "Invalid event format".to_string()
                    ))?
                    .clone();
                let result =
                    serde_json::from_value::<ExecutionResult>(execution_result).map_err(|e| {
                        LivenetError::ClientError(format!(
                            "Failed to parse `execution_result`: {}",
                            e
                        ))
                    })?;
                log::debug(serde_json::to_string_pretty(&result).unwrap());
                Ok(Some(result))
            } else {
                Ok(None)
            }
        }
        None => Ok(None)
    }
}

/// The hash can be in two formats:
/// - `Version1`: Standard transaction hash
/// - `Deploy`: Deploy transaction hash
fn match_hash(event: &serde_json::Value, expected_hash: &str) -> bool {
    let hash_obj = match event.get("transaction_hash") {
        Some(ho) => ho,
        None => return false
    };

    hash_obj
        .get("Version1")
        .or_else(|| hash_obj.get("Deploy"))
        .and_then(|v| v.as_str())
        .map(|s| s == expected_hash)
        .unwrap_or_default()
}
