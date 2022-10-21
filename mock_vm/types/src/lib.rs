#![feature(trait_alias)]

mod address;
pub mod contract_def;

pub type Balance = casper_types::U512;
pub type BlockTime = u64;
pub type CallArgs = casper_types::RuntimeArgs;
pub type Bytes = casper_types::bytesrepr::Bytes;
pub type TypedValue = casper_types::CLValue;
pub type Type = casper_types::CLType;
pub use casper_types::CLTyped;
pub use casper_types::bytesrepr::ToBytes;
pub use casper_types::bytesrepr::FromBytes;

pub use address::Address;
pub use odra_types;

pub trait OdraType: casper_types::CLTyped + casper_types::bytesrepr::ToBytes + casper_types::bytesrepr::FromBytes {}

macro_rules! impl_odra_type {
    ( $( $ty:ty ),+ ) => {
        $( impl OdraType for $ty {} )+
    };
}

impl_odra_type!(u8, u32, u64, i32, i64, bool, (), String);
