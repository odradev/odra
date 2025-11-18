//! Transaction watcher for monitoring SSE events stream.

use crate::error::LivenetError;
use crate::error::LivenetError::{ClientError, ExecutionError};
use crate::log;
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value as JsonValue;
use std::time::{Duration, Instant};

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

        self.process_stream(response.bytes_stream(), transaction_hash)
            .await
    }

    async fn process_stream(
        &self,
        mut stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        let start_time = Instant::now();
        let mut buffer = Vec::new();
        let mut event_data_lines = Vec::new();

        loop {
            // Calculate remaining timeout duration
            let elapsed = start_time.elapsed();
            if elapsed >= self.timeout {
                return Err(ExecutionError(String::from(
                    "Timeout waiting for transaction to be processed."
                )));
            }
            let remaining_duration = self.timeout.saturating_sub(elapsed);

            // Wrap stream.next() with timeout to prevent hanging
            let chunk_result = tokio::time::timeout(remaining_duration, stream.next()).await;

            let chunk = match chunk_result {
                Ok(Some(chunk)) => chunk,
                Ok(None) => {
                    // Stream ended - process any remaining accumulated data lines
                    return self.process_remaining_data(event_data_lines, transaction_hash);
                }
                Err(_) => {
                    // Timeout occurred
                    return Err(ExecutionError(String::from(
                        "Timeout waiting for transaction to be processed."
                    )));
                }
            };

            let bytes = chunk.map_err(|e| ClientError(format!("Error reading stream: {}", e)))?;
            buffer.extend_from_slice(&bytes);

            // Process complete lines (SSE format: "data: ...\n")
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                let line_str = match std::str::from_utf8(&line) {
                    Ok(s) => s.trim(),
                    Err(_) => continue
                };

                // Blank line separates SSE events - process accumulated data lines
                if line_str.is_empty() {
                    if !event_data_lines.is_empty() {
                        if self.process_event(&event_data_lines, transaction_hash)? {
                            return Ok(true);
                        }
                        event_data_lines.clear();
                    }
                    continue;
                }

                // Accumulate "data:" lines for multi-line events
                if line_str.starts_with("data:") {
                    let data_content = line_str.strip_prefix("data:").unwrap_or("").trim();
                    if !data_content.is_empty() {
                        event_data_lines.push(data_content.to_string());
                    }
                }
            }
        }
    }

    fn process_event(
        &self,
        event_data_lines: &[String],
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        // Join all accumulated data lines and parse as single JSON
        let json_str = event_data_lines.join("\n");

        // Parse and check for TransactionProcessed event
        let parsed: JsonValue = serde_json::from_str(&json_str)
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

    fn process_remaining_data(
        &self,
        event_data_lines: Vec<String>,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        if event_data_lines.is_empty() {
            return Ok(false);
        }

        self.process_event(&event_data_lines, transaction_hash)
    }
}
