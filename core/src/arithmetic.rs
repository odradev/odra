//! Safe, overflowing addition and subtraction utilities.
use crate::prelude::*;
use casper_types::{U128, U256, U512};

/// Overflowing addition, returning the result of addition or [ArithmeticsError::AdditionOverflow].
pub trait OverflowingAdd: Sized {
    /// Overflowing addition. Compute `self + rhs`, returns the result or error if overflowed.
    fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError>;
}

/// Overflowing subtraction, returning the result of addition or [ArithmeticsError::SubtractingOverflow].
pub trait OverflowingSub: Sized {
    /// Overflowing subtraction. Compute `self - rhs`, returns the result or error if overflowed.
    fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError>;
}

/// Computation result error.
#[cfg_attr(debug_assertions, derive(Debug, PartialEq, Eq))]
pub enum ArithmeticsError {
    /// Addition result exceeds the max value.
    AdditionOverflow,
    /// Subtraction result is lower than the min value.
    SubtractingOverflow,
    /// Conversion error
    ConversionError
}

/// Implements [OverflowingAdd] and [OverflowingSub] for all the given types.
macro_rules! impl_overflowing_add_sub {
    ( $( $ty:ty ),+ ) => {
        $(
            impl OverflowingAdd for $ty {
                fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.overflowing_add(rhs);
                    match is_overflowed {
                        true => Err(ArithmeticsError::AdditionOverflow.into()),
                        false => Ok(res)
                    }
                }
            }

            impl OverflowingSub for $ty {
                fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.overflowing_sub(rhs);
                    match is_overflowed {
                        true => Err(ArithmeticsError::SubtractingOverflow.into()),
                        false => Ok(res)
                    }
                }
            }
        )+
    };
}

impl_overflowing_add_sub!(u8, u16, u32, u64, i8, i16, i32, i64, U128, U256, U512);

/// Builds a [U256] from a `u128` in a const context.
///
/// [U256] is four 64-bit limbs, little-endian, so a `u128` fills the low two
/// and leaves the high two zero. `U256::from` cannot be used here because it is
/// not a `const fn`, which is the whole point: this can initialise a `const` or
/// `static`, where a runtime conversion cannot.
///
/// ```
/// use odra_core::prelude::u256;
/// use odra_core::casper_types::U256;
///
/// const ONE_TOKEN: U256 = u256(1_000_000_000_000_000_000);
/// assert_eq!(ONE_TOKEN, U256::from(1_000_000_000_000_000_000u64));
/// ```
pub const fn u256(n: u128) -> U256 {
    U256([n as u64, (n >> 64) as u64, 0, 0])
}

#[cfg(test)]
mod test {
    use crate::arithmetic::{u256, ArithmeticsError, OverflowingSub};
    use casper_types::U256;

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

    #[test]
    fn test_u256_zero() {
        assert_eq!(u256(0), U256::zero());
    }

    #[test]
    fn test_u256_small_value() {
        assert_eq!(u256(42), U256::from(42));
    }

    #[test]
    fn test_u256_above_u64_max() {
        let n = u128::from(u64::MAX) + 1;
        assert_eq!(u256(n), U256::from(u64::MAX) + U256::from(1));
    }

    #[test]
    fn test_u256_u128_max() {
        assert_eq!(u256(u128::MAX), U256::from(u128::MAX));
    }

    /// The point of the function is that it works in a `const` context, and
    /// nothing above actually checks that -- every test would still pass with
    /// `const` removed from the signature. A `const` item forces the compiler
    /// to evaluate it at compile time, so this fails to build rather than
    /// fails to assert if the constness is ever lost.
    #[test]
    fn test_u256_is_usable_in_a_const_context() {
        const ZERO: U256 = u256(0);
        const HIGH_LIMB: U256 = u256(1 << 64);
        const MAX: U256 = u256(u128::MAX);

        assert_eq!(ZERO, U256::zero());
        assert_eq!(HIGH_LIMB, U256::from(u64::MAX) + U256::from(1));
        assert_eq!(MAX, U256::from(u128::MAX));
    }
}
