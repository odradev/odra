mod casper_address;
pub mod contract_def;

pub use odra_types;

pub type Balance = casper_types::U512;
pub type BlockTime = u64;
pub type CallArgs = casper_types::RuntimeArgs;
pub type Bytes = casper_types::bytesrepr::Bytes;
pub type TypedValue = casper_types::CLValue;
pub type Type = casper_types::CLType;
// TODO: Remove ToBytes, FromBytes, CLTyped;
pub use casper_types::bytesrepr::Error as BytesreprError;
pub use casper_types::bytesrepr::FromBytes;
pub use casper_types::bytesrepr::ToBytes;
pub use casper_types::CLTyped;

pub use casper_address::CasperAddress as Address;

pub trait OdraType: CLTyped + ToBytes + FromBytes {}

macro_rules! impl_odra_type {
    ( $( $ty:ty ),+ ) => {
        $( impl OdraType for $ty {} )+
    };
}

impl_odra_type!(u8, u32, u64, i32, i64, bool, (), String, Bytes);
