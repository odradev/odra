use std::ops::{Add, Sub};
use std::ops::{Deref, DerefMut};

use odra_types::{
    arithmetic::{OverflowingAdd, OverflowingSub},
    ExecutionError
};

mod address;
mod types;
pub mod contract_def;

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

pub use address::Address;
pub use odra_types;

pub trait OdraType: ToBytes + FromBytes + CLTyped {}

// pub enum MockVMSerializationError {

// } 

// pub trait MockVMType: Sized {
//     fn ser(&self) -> Result<Vec<u8>, String>;
//     fn deser(bytes: Vec<u8>) -> Result<Self, MockVMSerializationError>;
// }

macro_rules! impl_odra_type {
    ( $( $ty:ty ),+ ) => {
        $( impl OdraType for $ty {} )+
    };
}

// u8, u32, u64, i32, i64, bool, (), String, U256, U512, (T, T), (T, T, T), Vec<T>, Result<T, T>, Option<T>

impl_odra_type!(u8, u32, u64, i32, i64, bool, (), String, U256, (Address, Address), Bytes);

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct U256 {
    inner: casper_types::U256
}

impl ToBytes for U256 {
    fn to_bytes(&self) -> Result<Vec<u8>, BytesreprError> {
        self.inner.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.inner.serialized_length()
    }
}

impl FromBytes for U256 {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), BytesreprError> {
        let (inner, bytes) = casper_types::U256::from_bytes(bytes)?;
        Ok((Self { inner }, bytes))
    }
}

impl CLTyped for U256 {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::U256
    }
}

impl Deref for U256 {
    type Target = casper_types::U256;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for U256 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Add for U256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner
        }
    }
}

impl Sub for U256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner - rhs.inner
        }
    }
}

impl OverflowingAdd for U256 {
    fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError> {
        Ok(self + rhs)
    }
}

impl OverflowingSub for U256 {
    fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError> {
        Ok(self - rhs)
    }
}

impl From<u32> for U256 {
    fn from(v: u32) -> Self {
        Self {
            inner: casper_types::U256::from(v)
        }
    }
}
