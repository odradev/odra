//! Server-Sent Events (SSE) stream parser.
//! Handles low-level parsing of SSE format from HTTP streams.

use crate::error::LivenetError;
use crate::error::LivenetError::ClientError;
use bytes::Bytes;
use std::fmt;

/// Represents a complete SSE event with its data content.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub data: String
}

impl SseEvent {
    pub fn new(data: String) -> Self {
        Self { data }
    }
}

/// Parses SSE events from a byte stream.
pub struct SseParser {
    buffer: String,
    current_event_data: String
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            current_event_data: String::new()
        }
    }

    /// Processes incoming bytes and yields all complete SSE events parsed from the chunk.
    pub async fn process_chunk(
        &mut self,
        chunk: Result<Bytes, reqwest::Error>
    ) -> Result<Vec<SseEvent>, LivenetError> {
        let bytes = chunk.map_err(|e| ClientError(format!("Error reading stream: {}", e)))?;
        self.buffer.push_str(&String::from_utf8_lossy(&bytes));

        let mut events = Vec::new();
        while let Some(event) = self.extract_next_event()? {
            events.push(event);
        }
        Ok(events)
    }

    /// Extracts the next complete event from the buffer.
    fn extract_next_event(&mut self) -> Result<Option<SseEvent>, LivenetError> {
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim().to_string();
            self.buffer.replace_range(..=newline_pos, "");

            if line.is_empty() {
                // Blank line marks the end of an event
                if !self.current_event_data.is_empty() {
                    let event = SseEvent::new(self.current_event_data.clone());
                    self.current_event_data.clear();
                    return Ok(Some(event));
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                // Accumulate data lines for multi-line events
                self.current_event_data.push_str(data.trim());
                self.current_event_data.push('\n');
            }
        }

        Ok(None)
    }

    /// Returns any remaining event data when the stream ends.
    pub fn finalize(&mut self) -> Option<SseEvent> {
        if self.current_event_data.is_empty() {
            None
        } else {
            Some(SseEvent::new(self.current_event_data.clone()))
        }
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SseEvent(data: {})", self.data)
    }
}
