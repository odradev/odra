//! Events interface and errors.

use alloc::string::String;

#[cfg(not(target_arch = "wasm32"))]
use crate::contract_def::Event as Schema;

/// Event interface
pub trait OdraEvent {
    /// Emits &self in the current environment.
    fn emit(self);
    /// Returns the event name.
    fn name() -> String;
    #[cfg(not(target_arch = "wasm32"))]
    /// Returns the event schema.
    fn schema() -> Schema;
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
