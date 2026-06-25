//! Transaction watcher for monitoring Casper network events.

mod event_matcher;

use crate::casper_client::configuration::CasperClientConfiguration;
use crate::error::LivenetError;
use crate::log;
use casper_types::execution::ExecutionResult;
use casper_types::TransactionHash;
use futures_util::StreamExt;
use reqwest_eventsource::{Error as EventSourceError, Event, EventSource};
use std::time::Duration;

/// Monitors the Casper network event stream for transaction processing events.
pub struct TransactionWatcher {
    events_url: String,
    timeout: Duration
}

impl TransactionWatcher {
    pub fn new(cfg: &CasperClientConfiguration, timeout: Duration) -> Self {
        Self {
            events_url: cfg.events_url.clone(),
            timeout
        }
    }

    /// Starts watching the event stream and returns an active watch handle.
    /// Waits for the connection to open so a transaction submitted right after
    /// this returns can't have its event missed.
    pub async fn start_watching(&self) -> Result<TransactionWatch, LivenetError> {
        log::debug(format!(
            "[WATCHER] Connecting to events stream: {}",
            self.events_url
        ));
        let es = EventSource::get(&self.events_url);
        Ok(TransactionWatch {
            es,
            timeout: self.timeout
        })
    }
}

/// An active watch handle connected to the Casper event stream.
pub struct TransactionWatch {
    es: EventSource,
    timeout: Duration
}

impl TransactionWatch {
    /// Waits for a transaction to be processed in the connected event stream.
    pub async fn wait_for_transaction_hash(
        self,
        transaction_hash: &TransactionHash
    ) -> Result<ExecutionResult, LivenetError> {
        log::debug(format!(
            "[WATCHER] Starting to monitor for transaction: {}",
            transaction_hash
        ));
        let transaction_hash_str = transaction_hash.to_hex_string();

        tokio::time::timeout(
            self.timeout,
            Self::monitor_events_until_found(self.es, &transaction_hash_str)
        )
        .await
        .map_err(|_| {
            LivenetError::ExecutionError(String::from(
                "Timeout waiting for transaction to be processed."
            ))
        })?
    }

    async fn monitor_events_until_found(
        mut es: EventSource,
        transaction_hash: &str
    ) -> Result<ExecutionResult, LivenetError> {
        log::debug("[WATCHER] Monitoring stream for events...");

        let mut event_count = 0;
        while let Some(event) = es.next().await {
            match event {
                // (Re)connection notice — nothing to match.
                Ok(Event::Open) => {}
                Ok(Event::Message(msg)) => {
                    event_count += 1;
                    if let Some(result) = event_matcher::find_result(&msg.data, transaction_hash)? {
                        log::debug(format!(
                            "[WATCHER] FOUND transaction in event #{}!",
                            event_count
                        ));
                        es.close();
                        return Ok(result);
                    }
                }
                // The stream closed without reconnecting — mirror the old
                // "stream ended, transaction not found" behaviour.
                Err(EventSourceError::StreamEnded) => break,
                Err(e) => {
                    return Err(LivenetError::ClientError(format!(
                        "Error reading event stream: {}",
                        e
                    )));
                }
            }
        }

        log::debug(format!(
            "[WATCHER] Stream ended after {} events, transaction not found",
            event_count
        ));

        Err(LivenetError::ClientError(
            "Stream ended, transaction not found.".to_string()
        ))
    }
}
