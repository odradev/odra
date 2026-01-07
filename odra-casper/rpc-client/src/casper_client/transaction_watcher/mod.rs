//! Transaction watcher for monitoring Casper network events.

mod event_matcher;
mod sse_parser;

use crate::error::LivenetError;
use crate::error::LivenetError::{ClientError, ExecutionError};
use crate::log;
use event_matcher::EventMatcher;
use futures_util::StreamExt;
use reqwest::Client;
use sse_parser::SseParser;
use std::pin::Pin;
use std::time::Duration;

/// Type alias for the boxed byte stream from the SSE connection.
type ByteStream =
    Pin<Box<dyn futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>;

/// Monitors the Casper network event stream for transaction processing events.
pub struct TransactionWatcher {
    events_url: String,
    timeout: Duration
}

impl TransactionWatcher {
    pub fn new(events_url: String, timeout: Duration) -> Self {
        Self {
            events_url,
            timeout
        }
    }

    /// Starts watching the event stream and returns an active watch handle.
    /// Call this BEFORE sending the transaction to ensure no events are missed.
    pub async fn start_watching(&self) -> Result<TransactionWatch, LivenetError> {
        let stream = self.connect_to_event_stream().await?;
        Ok(TransactionWatch {
            stream,
            timeout: self.timeout
        })
    }

    /// Connects to the Casper node's event stream.
    async fn connect_to_event_stream(&self) -> Result<ByteStream, LivenetError> {
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

        Ok(Box::pin(response.bytes_stream()))
    }
}

/// An active watch handle connected to the Casper event stream.
pub struct TransactionWatch {
    stream: ByteStream,
    timeout: Duration
}

impl TransactionWatch {
    /// Waits for a transaction to be processed in the connected event stream.
    pub async fn wait_for_transaction_hash(
        self,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        log::wait(format!(
            "Waiting for transaction {:?} to be processed.",
            transaction_hash
        ));

        tokio::time::timeout(
            self.timeout,
            Self::monitor_events_until_found(self.stream, transaction_hash)
        )
        .await
        .map_err(|_| {
            ExecutionError(String::from(
                "Timeout waiting for transaction to be processed."
            ))
        })?
    }

    /// Monitors the event stream until the transaction is found or the stream ends.
    async fn monitor_events_until_found(
        mut stream: ByteStream,
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
