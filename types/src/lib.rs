mod address;
mod error;
pub mod event;

pub use address::Address;
pub use casper_types::*;
pub type EventData = Vec<u8>;
pub use error::{OdraError, VmError};


pub trait ToBytes: Sized {
    type Error;
    
    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
}

pub trait FromBytes: Sized {
    type Error;
    type Item;

    fn deserialize(data: Vec<u8>) -> Result<(Self::Item, Vec<u8>), Self::Error>;
}