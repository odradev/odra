//! Safe, overflowing addition and subtraction utilities.
use crate::ExecutionError;

/// Overflowing addition, returning the result of addition or [ArithmeticsError::AdditionOverflow].
pub trait OverflowingAdd: Sized {
    /// Overflowing addition. Compute `self + rhs`, returns the result or error if overflowed.
    fn overflowing_add(self, rhs: &Self) -> Result<Self, ExecutionError>;
}

/// Overflowing subtraction, returning the result of addition or [ArithmeticsError::SubtractingOverflow].
pub trait OverflowingSub: Sized {
    /// Overflowing subtraction. Compute `self - rhs`, returns the result or error if overflowed.
    fn overflowing_sub(self, rhs: &Self) -> Result<Self, ExecutionError>;
}

/// Computation result error.
#[cfg_attr(debug_assertions, derive(Debug, PartialEq, Eq))]
pub enum ArithmeticsError {
    // Addition result exceeds the max value.
    AdditionOverflow,
    // Subtraction result is lower than the min value.
    SubtractingOverflow
}

/// Implements [OverflowingAdd] and [OverflowingSub] for all the given types.
#[macro_export]
macro_rules! impl_overflowing_add_sub {
    ( $( $ty:ty ),+ ) => {
        $(
            impl OverflowingAdd for $ty {
                fn overflowing_add(self, rhs: &Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.overflowing_add(*rhs);
                    match is_overflowed {
                        true => Err(ArithmeticsError::AdditionOverflow.into()),
                        false => Ok(res)
                    }
                }
            }

            impl OverflowingSub for $ty {
                fn overflowing_sub(self, rhs: &Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.overflowing_sub(*rhs);
                    match is_overflowed {
                        true => Err(ArithmeticsError::SubtractingOverflow.into()),
                        false => Ok(res)
                    }
                }
            }
        )+
    };
}

impl_overflowing_add_sub!(u8, u16, u32, u64, i8, i16, i32, i64);

#[cfg(test)]
mod test {
    use crate::arithmetic::{ArithmeticsError, OverflowingSub};

    use super::OverflowingAdd;

    #[test]
    fn test_add() {
        assert_eq!(<u8 as OverflowingAdd>::overflowing_add(2u8, &1u8), Ok(3u8));
        assert_eq!(
            <u8 as OverflowingAdd>::overflowing_add(u8::MAX, &1u8),
            Err(ArithmeticsError::AdditionOverflow.into())
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(<u8 as OverflowingSub>::overflowing_sub(2u8, &1u8), Ok(1u8));
        assert_eq!(
            <u8 as OverflowingSub>::overflowing_sub(u8::MIN, &1u8),
            Err(ArithmeticsError::SubtractingOverflow.into())
        );
    }
}
