//! Events interface and errors.

use crate::contract_def::Event;

/// Event interface
pub trait OdraEvent {
    /// Emits &self in the current environment.
    fn emit(self);
    /// Returns the event name.
    fn name() -> String;
    /// Returns the event schema.
    fn schema() -> Vec<Event>;
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
    Parsing
}