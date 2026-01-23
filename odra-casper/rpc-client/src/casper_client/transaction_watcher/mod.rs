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
    /// Waits for the first bytes to ensure the connection is fully established.
    pub async fn start_watching(&self) -> Result<TransactionWatch, LivenetError> {
        log::debug(format!(
            "[WATCHER] Connecting to events stream: {}",
            self.events_url
        ));
        let stream = self.connect_to_event_stream().await?;
        log::debug("[WATCHER] HTTP connection established, waiting for first bytes...");

        // Wait for the first bytes to ensure stream is ready
        let (stream, initial_bytes) = Self::wait_for_first_bytes(stream).await?;

        let bytes_len = initial_bytes.as_ref().map(|b| b.len()).unwrap_or(0);
        log::debug(format!(
            "[WATCHER] Received first {} bytes, stream ready!",
            bytes_len
        ));

        Ok(TransactionWatch {
            stream,
            initial_bytes,
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

    /// Waits for the first bytes from the stream to confirm the connection is ready.
    async fn wait_for_first_bytes(
        mut stream: ByteStream
    ) -> Result<(ByteStream, Option<bytes::Bytes>), LivenetError> {
        let first_chunk = stream.next().await;

        match first_chunk {
            Some(Ok(bytes)) => Ok((stream, Some(bytes))),
            Some(Err(e)) => Err(ClientError(format!(
                "Error reading first bytes from stream: {}",
                e
            ))),
            None => Err(ClientError(
                "Events stream closed before receiving any data".to_string()
            ))
        }
    }
}

/// An active watch handle connected to the Casper event stream.
pub struct TransactionWatch {
    stream: ByteStream,
    initial_bytes: Option<bytes::Bytes>,
    timeout: Duration
}

impl TransactionWatch {
    /// Waits for a transaction to be processed in the connected event stream.
    pub async fn wait_for_transaction_hash(
        self,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        log::debug(format!(
            "[WATCHER] Starting to monitor for transaction: {}",
            transaction_hash
        ));

        tokio::time::timeout(
            self.timeout,
            Self::monitor_events_until_found(self.stream, self.initial_bytes, transaction_hash)
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
        initial_bytes: Option<bytes::Bytes>,
        transaction_hash: &str
    ) -> Result<bool, LivenetError> {
        let mut parser = SseParser::new();
        let mut event_count = 0;

        // Process initial bytes first (received while confirming connection)
        if let Some(bytes) = initial_bytes {
            log::debug(format!(
                "[WATCHER] Processing {} initial buffered bytes",
                bytes.len()
            ));
            let events = parser.process_chunk(Ok(bytes)).await?;
            for event in events {
                event_count += 1;
                log::debug(format!(
                    "[WATCHER] Event #{}: checking if matches...",
                    event_count
                ));
                if EventMatcher::matches_transaction_hash(&event.data, transaction_hash)? {
                    log::debug(format!(
                        "[WATCHER] FOUND transaction in event #{}!",
                        event_count
                    ));
                    return Ok(true);
                }
            }
        }

        log::debug("[WATCHER] Monitoring stream for events...");

        while let Some(chunk) = stream.next().await {
            let events = parser.process_chunk(chunk).await?;
            for event in events {
                event_count += 1;
                if event_count <= 5 || event_count % 10 == 0 {
                    log::debug(format!(
                        "[WATCHER] Event #{}: received, checking...",
                        event_count
                    ));
                }
                if EventMatcher::matches_transaction_hash(&event.data, transaction_hash)? {
                    log::debug(format!(
                        "[WATCHER] FOUND transaction in event #{}!",
                        event_count
                    ));
                    return Ok(true);
                }
            }
        }

        log::debug(format!(
            "[WATCHER] Stream ended after {} events, transaction not found",
            event_count
        ));

        if let Some(event) = parser.finalize() {
            return EventMatcher::matches_transaction_hash(&event.data, transaction_hash);
        }

        Ok(false)
    }
}
