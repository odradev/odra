mod args;
mod bytes;
mod casper_address;
mod ty;

pub use args::CallArgs;
pub use bytes::Bytes;
pub use ty::Typed;

pub use casper_address::CasperAddress as Address;

pub type Balance = casper_types::U512;
pub type BlockTime = u64;
pub use casper_types::{
    bytesrepr::{Error as BytesError, FromBytes, ToBytes},
    CLType, CLTyped,
};
pub trait OdraType: CLTyped + ToBytes + FromBytes {}

impl<T: CLTyped + ToBytes + FromBytes> OdraType for T {}
