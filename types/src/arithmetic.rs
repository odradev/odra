use casper_types::{U128, U256, U512};

use crate::OdraError;

pub trait OverflowingAdd: Sized {
    fn overflowing_add(self, rhs: Self) -> Result<Self, OdraError>;
}

pub trait OverflowingSub: Sized {
    fn overflowing_sub(self, rhs: Self) -> Result<Self, OdraError>;
}

macro_rules! impl_overflowing_add {
    ( $( $ty:ty ),+ ) => {
        $( impl OverflowingAdd for $ty {
            fn overflowing_add(self, rhs: Self) -> Result<Self, OdraError> {
                let (res, is_overflowed)  = self.overflowing_add(rhs);
                match is_overflowed {
                    true => Err(ArithmeticsError::AdditionOverflow.into()),
                    false => Ok(res)
                }
            }
        })+
    };
}

macro_rules! impl_overflowing_sub {
    ( $( $ty:ty ),+ ) => {
        $( impl OverflowingSub for $ty {
            fn overflowing_sub(self, rhs: Self) -> Result<Self, OdraError> {
                let (res, is_overflowed)  = self.overflowing_sub(rhs);
                match is_overflowed {
                    true => Err(ArithmeticsError::SubtractingOverflow.into()),
                    false => Ok(res)
                }
            }
        })+
    };
}

impl_overflowing_add!(u8, u16, u32, u64, U128, U256, U512, i8, i16, i32, i64);
impl_overflowing_sub!(u8, u16, u32, u64, U128, U256, U512, i8, i16, i32, i64);

pub enum ArithmeticsError {
    AdditionOverflow,
    SubtractingOverflow,
}
