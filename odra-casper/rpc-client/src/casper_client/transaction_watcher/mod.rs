//! Transaction watcher for monitoring Casper network events.
//!
//! The `TransactionWatcher` monitors the Casper network's event stream
//! and notifies when a specific transaction has been processed.
//!
//! # Example
//!
//! ```no_run
//! use std::time::Duration;
//! use odra_casper_rpc_client::casper_client::transaction_watcher::TransactionWatcher;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let watcher = TransactionWatcher::new(
//!     "http://localhost:18101/events".to_string(),
//!     Duration::from_secs(60)
//! );
//!
//! let found = watcher.wait_for_transaction_hash("abc123...").await?;
//! if found {
//!     println!("Transaction was processed!");
//! }
//! # Ok(())
//! # }
//! ```

mod event_matcher;
mod sse_parser;

use crate::error::LivenetError;
use crate::error::LivenetError::{ClientError, ExecutionError};
use crate::log;
use event_matcher::EventMatcher;
use futures_util::StreamExt;
use reqwest::Client;
use sse_parser::SseParser;
use std::time::Duration;

/// Monitors the Casper network event stream for transaction processing events.
///
/// A `TransactionWatcher` connects to the Casper node's event stream and
/// watches for when a specific transaction is processed. This is useful
/// for waiting for deploy or call transactions to complete.
pub struct TransactionWatcher {
    events_url: String,
    timeout: Duration
}

impl TransactionWatcher {
    /// Creates a new transaction watcher.
    ///
    /// # Arguments
    ///
    /// * `events_url` - The URL of the Casper node's events stream endpoint
    /// * `timeout` - Maximum time to wait for the transaction to be processed
    pub fn new(events_url: String, timeout: Duration) -> Self {
        Self {
            events_url,
            timeout
        }
    }

    /// Waits for a transaction to be processed by monitoring the events stream.
    ///
    /// This method connects to the Casper network's event stream and monitors
    /// incoming events until it finds a `TransactionProcessed` event matching
    /// the provided transaction hash.
    ///
    /// # Arguments
    ///
    /// * `transaction_hash` - The hash of the transaction to wait for
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - The transaction was found and processed
    /// * `Ok(false)` - The stream ended without finding the transaction
    /// * `Err(...)` - An error occurred (timeout, connection failure, etc.)
    pub async fn wait_for_transaction_hash(
        &self,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        log::wait(format!(
            "Waiting for transaction {:?} to be processed.",
            transaction_hash
        ));

        let event_stream = self.connect_to_event_stream().await?;

        tokio::time::timeout(
            self.timeout,
            self.monitor_events_until_found(event_stream, transaction_hash)
        )
        .await
        .map_err(|_| {
            ExecutionError(String::from(
                "Timeout waiting for transaction to be processed."
            ))
        })?
    }

    /// Connects to the Casper node's event stream.
    async fn connect_to_event_stream(
        &self
    ) -> Result<
        impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
        LivenetError
    > {
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

        Ok(response.bytes_stream())
    }

    /// Monitors the event stream until the transaction is found or the stream ends.
    async fn monitor_events_until_found(
        &self,
        mut stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        let mut parser = SseParser::new();

        while let Some(chunk) = stream.next().await {
            if let Some(event) = parser.process_chunk(chunk).await? {
                if EventMatcher::matches_transaction_hash(&event.data, transaction_hash)? {
                    return Ok(true);
                }
            }
        }

        // Check for any remaining event data when stream ends
        if let Some(event) = parser.finalize() {
            return EventMatcher::matches_transaction_hash(&event.data, transaction_hash);
        }

        Ok(false)
    }
}
