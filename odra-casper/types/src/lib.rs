mod casper_address;
pub mod contract_def;

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};
pub use odra_types;

pub type Balance = casper_types::U512;
pub type BlockTime = u64;
pub type CallArgs = casper_types::RuntimeArgs;
pub type Bytes = casper_types::bytesrepr::Bytes;
pub type TypedValue = casper_types::CLValue;
pub type Type = casper_types::CLType;

pub use casper_address::CasperAddress as Address;

pub trait OdraType: CLTyped + ToBytes + FromBytes {}
