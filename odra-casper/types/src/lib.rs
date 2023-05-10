mod args;
mod casper_address;
mod ty;
mod uints;
pub use args::CallArgs;
pub use casper_types;
pub use casper_types::bytesrepr::Bytes;
pub use ty::Typed;
pub use uints::{U128, U256, U512};

pub use casper_address::CasperAddress as Address;
/// A type representing the amount of native tokens.
pub type Balance = U512;
/// A type representing the block time.
pub type BlockTime = u64;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

/// A type that can be written to the storage and read from the storage.
pub trait OdraType: CLTyped + ToBytes + FromBytes {
    fn serialize(&self) -> Option<Vec<u8>> {
        self.to_bytes().ok()
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        match Self::from_bytes(bytes) {
            Ok((result, leftover)) => {
                if leftover.is_empty() {
                    Some(result)
                } else {
                    None
                }
            }
            Err(_) => None
        }
    }
}

impl<T: CLTyped + ToBytes + FromBytes> OdraType for T {}

/// Make sure the type of a value is not [`CLType::Any`](casper_types::CLType::Any).
pub fn validate_type<T: casper_types::CLTyped>(
    t: &T
) -> Result<(), casper_types::bytesrepr::Error> {
    casper_event_standard::validate_type(t)
}
