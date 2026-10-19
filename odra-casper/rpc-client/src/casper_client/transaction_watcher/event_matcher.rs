//! Transaction event matcher.
//! Handles matching transaction hashes in Casper network events.

use crate::{error::LivenetError, log};
use casper_types::contract_messages::Messages;
use casper_types::execution::ExecutionResult;

/// What the node reports about a processed transaction: its execution result and the messages
/// (native events) it emitted. The messages live only here, not in the global state.
#[derive(Debug)]
pub struct ProcessedTransaction {
    /// The execution result.
    pub execution_result: ExecutionResult,
    /// The messages emitted by the transaction, in emission order.
    pub messages: Messages
}

/// Finds the [ProcessedTransaction] with the specified transaction hash.
///
/// # Arguments
///
/// * `event_json` - The JSON string of the SSE event
/// * `expected_hash` - The transaction hash we're looking for
pub fn find_result(
    event_json: &str,
    expected_hash: &str
) -> Result<Option<ProcessedTransaction>, LivenetError> {
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
                // Absent in a legacy (deploy) event; then there are no messages either.
                let messages = match tx.get("messages") {
                    Some(messages) => serde_json::from_value::<Messages>(messages.clone())
                        .map_err(|e| {
                            LivenetError::ClientError(format!("Failed to parse `messages`: {}", e))
                        })?,
                    None => Messages::new()
                };
                Ok(Some(ProcessedTransaction {
                    execution_result: result,
                    messages
                }))
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

#[cfg(test)]
mod tests {
    use super::find_result;
    use casper_types::bytesrepr::Bytes;
    use casper_types::contract_messages::{Message, MessagePayload, TopicNameHash};
    use casper_types::execution::{Effects, ExecutionResult, ExecutionResultV2};
    use casper_types::{account::AccountHash, EntityAddr, Gas, InitiatorAddr, U512};

    const HASH: &str = "0101010101010101010101010101010101010101010101010101010101010101";

    fn execution_result() -> ExecutionResult {
        ExecutionResult::V2(Box::new(ExecutionResultV2 {
            initiator: InitiatorAddr::AccountHash(AccountHash::new([7; 32])),
            error_message: None,
            current_price: 1,
            limit: Gas::zero(),
            consumed: Gas::zero(),
            cost: U512::zero(),
            refund: U512::zero(),
            transfers: vec![],
            size_estimate: 0,
            effects: Effects::new()
        }))
    }

    fn event(messages: Option<serde_json::Value>) -> String {
        let mut tx = serde_json::json!({
            "transaction_hash": { "Version1": HASH },
            "execution_result": execution_result()
        });
        if let Some(messages) = messages {
            tx["messages"] = messages;
        }
        serde_json::json!({ "TransactionProcessed": tx }).to_string()
    }

    #[test]
    fn messages_of_the_processed_transaction_are_kept() {
        let entity = EntityAddr::SmartContract([3; 32]);
        let payload = MessagePayload::Bytes(Bytes::from(vec![1u8, 2, 3]));
        let message = Message::new(
            entity,
            payload.clone(),
            "odra_events".to_string(),
            TopicNameHash::new([0x4d; 32]),
            0,
            0
        );
        let event = event(Some(serde_json::to_value(vec![message]).unwrap()));

        let processed = find_result(&event, HASH).unwrap().expect("hash matches");
        assert_eq!(processed.messages.len(), 1);
        assert_eq!(processed.messages[0].entity_addr(), &entity);
        assert_eq!(processed.messages[0].payload(), &payload);
    }

    #[test]
    fn an_event_without_messages_has_none() {
        let processed = find_result(&event(None), HASH)
            .unwrap()
            .expect("hash matches");
        assert!(processed.messages.is_empty());
    }

    #[test]
    fn another_transaction_is_skipped() {
        let other = "02".repeat(32);
        assert!(find_result(&event(None), &other).unwrap().is_none());
    }
}
