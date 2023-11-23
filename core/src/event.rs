//! Events interface and errors.

use crate::prelude::string::String;

/// Event-related errors.
#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub enum EventError {
    /// The type of event is different than expected.
    UnexpectedType(String),
    /// Index of the event is out of bounds.
    IndexOutOfBounds,
    /// Formatting error while deserializing.
    Formatting,
    /// Unexpected error while deserializing.
    Parsing,
    /// Could not extract event name.
    CouldntExtractName
}
