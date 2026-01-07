//! Transaction watcher for monitoring Casper network events.

mod event_matcher;
mod sse_parser;

use crate::error::LivenetError;
use crate::error::LivenetError::{ClientError, ExecutionError};
use crate::log;
use event_matcher::EventMatcher;
use futures_util::StreamExt;
use reqwest::Client;
use sse_parser::{SseEvent, SseParser};
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
    /// Waits for the first event to ensure the connection is fully established.
    pub async fn start_watching(&self) -> Result<TransactionWatch, LivenetError> {
        let stream = self.connect_to_event_stream().await?;

        // Wait for the first event to ensure stream is ready
        let (stream, buffered_events) = Self::wait_for_first_event(stream).await?;

        Ok(TransactionWatch {
            stream,
            buffered_events,
            timeout: self.timeout
        })
    }

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

    /// Waits for the first event from the stream to confirm the connection is ready.
    /// Returns the stream and any events received while waiting.
    async fn wait_for_first_event(
        mut stream: ByteStream
    ) -> Result<(ByteStream, Vec<SseEvent>), LivenetError> {
        let mut parser = SseParser::new();
        let mut buffered_events = Vec::new();

        // Wait for at least one event to confirm stream is active
        while let Some(chunk) = stream.next().await {
            if let Some(event) = parser.process_chunk(chunk).await? {
                buffered_events.push(event);
                break;
            }
        }

        if buffered_events.is_empty() {
            return Err(ClientError(
                "Events stream closed before receiving first event".to_string()
            ));
        }

        Ok((stream, buffered_events))
    }
}

/// An active watch handle connected to the Casper event stream.
pub struct TransactionWatch {
    stream: ByteStream,
    buffered_events: Vec<SseEvent>,
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
            Self::monitor_events_until_found(self.stream, self.buffered_events, transaction_hash)
        )
        .await
        .map_err(|_| {
            ExecutionError(String::from(
                "Timeout waiting for transaction to be processed."
            ))
        })?
    }

    async fn monitor_events_until_found(
        mut stream: ByteStream,
        buffered_events: Vec<SseEvent>,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        // First check buffered events (received while waiting for connection)
        for event in buffered_events {
            if EventMatcher::matches_transaction_hash(&event.data, transaction_hash)? {
                return Ok(true);
            }
        }

        let mut parser = SseParser::new();

        while let Some(chunk) = stream.next().await {
            if let Some(event) = parser.process_chunk(chunk).await? {
                if EventMatcher::matches_transaction_hash(&event.data, transaction_hash)? {
                    return Ok(true);
                }
            }
        }

        if let Some(event) = parser.finalize() {
            return EventMatcher::matches_transaction_hash(&event.data, transaction_hash);
        }

        Ok(false)
    }
}
