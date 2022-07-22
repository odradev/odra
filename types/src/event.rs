//! Events interface and errors

/// Event interface
pub trait Event {
    /// Emits &self in the current environment.
    fn emit(&self);
    /// Returns the event name.
    fn name(&self) -> String;
}

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
}

impl From<casper_types::bytesrepr::Error> for EventError {
    fn from(err: casper_types::bytesrepr::Error) -> Self {
        match err {
            casper_types::bytesrepr::Error::Formatting => Self::Formatting,
            _ => Self::Parsing,
        }
    }
}
