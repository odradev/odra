use casper_types::{U128, U256, U512};

use crate::ExecutionError;

pub trait OverflowingAdd: Sized {
    fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError>;
}

pub trait OverflowingSub: Sized {
    fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError>;
}

macro_rules! impl_overflowing_add {
    ( $( $ty:ty ),+ ) => {
        $( impl OverflowingAdd for $ty {
            fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError> {
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
            fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError> {
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

#[cfg(test)]
mod test {
    use crate::arithmetic::{ArithmeticsError, OverflowingSub};

    use super::OverflowingAdd;

    #[test]
    fn test_add() {
        assert_eq!(<u8 as OverflowingAdd>::overflowing_add(2u8, 1u8), Ok(3u8));
        assert_eq!(
            <u8 as OverflowingAdd>::overflowing_add(u8::MAX, 1u8),
            Err(ArithmeticsError::AdditionOverflow.into())
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(<u8 as OverflowingSub>::overflowing_sub(2u8, 1u8), Ok(1u8));
        assert_eq!(
            <u8 as OverflowingSub>::overflowing_sub(u8::MIN, 1u8),
            Err(ArithmeticsError::SubtractingOverflow.into())
        );
    }
}
