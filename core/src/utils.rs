use crate::event::EventError;
use crate::prelude::*;
use casper_event_standard::casper_types::bytesrepr::ToBytes;
use odra_types::{Bytes, FromBytes};

pub fn serialize<T: ToBytes>(value: &T) -> Bytes {
    Bytes::from(value.to_bytes().expect("Coulnd't serialize"))
}

/// Returns the name of the passed event
pub(crate) fn extract_event_name(bytes: &[u8]) -> Result<String, EventError> {
    let name: String = FromBytes::from_bytes(bytes)
        .map_err(|_| EventError::CouldntExtractName)?
        .0;
    name.strip_prefix("event_")
        .map(|s| s.to_string())
        .ok_or(EventError::UnexpectedType(name))
}
