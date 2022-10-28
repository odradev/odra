mod args;
mod bytes;
mod casper_address;
mod ty;

pub use args::CallArgs;
pub use bytes::Bytes;
pub use ty::Typed;

pub use casper_address::CasperAddress as Address;
use casper_types::{U256, U512};

pub type Balance = casper_types::U512;
pub type BlockTime = u64;
pub use casper_types::{
    bytesrepr::{Error as BytesError, FromBytes, ToBytes},
    CLType, CLTyped,
};

pub trait OdraType: CLTyped + ToBytes + FromBytes {}

macro_rules! impl_odra_type {
    ( $( $ty:ty ),+ ) => {
        $( impl OdraType for $ty {} )+
    };
}

impl_odra_type!(u8, u32, u64, i32, i64, bool, (), String, Bytes, U256, U512);
