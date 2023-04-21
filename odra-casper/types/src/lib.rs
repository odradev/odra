mod args;
mod casper_address;
mod ty;
mod uints;
pub use args::CallArgs;
pub use casper_types;
pub use casper_types::bytesrepr::Bytes;
pub use ty::Typed;
pub use uints::{U256, U512};

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
pub trait OdraType: CLTyped + ToBytes + FromBytes {}

impl<T: CLTyped + ToBytes + FromBytes> OdraType for T {}
