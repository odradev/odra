//! Transaction watcher for monitoring SSE events stream.

use crate::error::LivenetError;
use crate::error::LivenetError::{ClientError, ExecutionError};
use crate::log;
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Watches for transaction processed events in an SSE stream.
pub struct TransactionWatcher {
    events_url: String,
    timeout: Duration
}

impl TransactionWatcher {
    /// Creates a new transaction watcher.
    pub fn new(events_url: String, timeout: Duration) -> Self {
        Self {
            events_url,
            timeout
        }
    }

    /// Waits for a transaction to be processed by monitoring the events stream.
    ///
    /// Returns `Ok(true)` when the transaction hash is found in a TransactionProcessed event,
    /// `Ok(false)` if the stream ended without finding it, or an error on timeout or connection failure.
    pub async fn wait_for_transaction_hash(
        &self,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        log::wait(format!(
            "Waiting for transaction {:?} to be processed.",
            transaction_hash
        ));

        let client = Client::new();
        let response = client
            .get(&self.events_url)
            .send()
            .await
            .map_err(|e| ClientError(format!("Failed to connect to events stream: {}", e)))?;

        if !response.status().is_success() {
            return Err(ClientError(format!(
                "Events stream returned status: {}",
                response.status()
            )));
        }

        tokio::time::timeout(
            self.timeout,
            self.process_stream(response.bytes_stream(), transaction_hash)
        )
        .await
        .map_err(|_| {
            ExecutionError(String::from(
                "Timeout waiting for transaction to be processed."
            ))
        })?
    }

    async fn process_stream(
        &self,
        mut stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        let mut buffer = String::new();
        let mut event_data = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| ClientError(format!("Error reading stream: {}", e)))?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim().to_string();
                buffer.replace_range(..=newline_pos, "");

                if line.is_empty() {
                    if !event_data.is_empty() {
                        if self.check_event(&event_data, transaction_hash)? {
                            return Ok(true);
                        }
                        event_data.clear();
                    }
                } else if let Some(data) = line.strip_prefix("data:") {
                    event_data.push_str(data.trim());
                    event_data.push('\n');
                }
            }
        }

        // Check remaining data if stream ended
        if !event_data.is_empty() {
            return self.check_event(&event_data, transaction_hash);
        }

        Ok(false)
    }

    fn check_event(&self, json_str: &str, transaction_hash: &str) -> Result<bool, LivenetError> {
        let parsed: JsonValue = serde_json::from_str(json_str.trim())
            .map_err(|e| ClientError(format!("Failed to parse event JSON: {}", e)))?;

        if let Some(transaction_processed) = parsed.get("TransactionProcessed") {
            if let Some(hash_obj) = transaction_processed.get("transaction_hash") {
                let hash_str = hash_obj
                    .get("Version1")
                    .or_else(|| hash_obj.get("Deploy"))
                    .and_then(|v| v.as_str());

                if let Some(hash) = hash_str {
                    if hash == transaction_hash {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}
