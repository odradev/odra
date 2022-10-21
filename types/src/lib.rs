pub mod arithmetic;
mod error;
pub mod event;
pub mod token;

pub use error::{CollectionError, ExecutionError, OdraError, VmError};
/// Serialized event struct representation
pub type EventData = Vec<u8>;
