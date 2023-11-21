use crate::event::EventError;
use crate::prelude::*;
use odra_types::FromBytes;

/// Returns the name of the passed event
pub(crate) fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
    let name: String = FromBytes::from_bytes(bytes)
        .map_err(|_| EventError::CouldntExtractName)?
        .0;
    name.strip_prefix("event_")
        .map(|s| s.to_string())
        .ok_or(EventError::UnexpectedType(name))
}
