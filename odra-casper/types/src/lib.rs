mod args;
mod bytes;
mod casper_address;
mod ty;
mod uints;

pub use args::CallArgs;
pub use bytes::Bytes;
pub use ty::Typed;
pub use uints::{U256, U512};

pub use casper_address::CasperAddress as Address;
pub type Balance = U512;
pub type BlockTime = u64;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};
pub trait OdraType: CLTyped + ToBytes + FromBytes {}

impl<T: CLTyped + ToBytes + FromBytes> OdraType for T {}
