mod address;
mod error;

pub use address::Address;
pub use casper_types::*;
pub type EventData = Vec<u8>;
pub use error::OdraError;
