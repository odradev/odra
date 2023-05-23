use crate::{CLTyped, FromBytes, ToBytes};
use num_traits::{
    Bounded, CheckedMul, CheckedSub, Num, One, Unsigned, WrappingAdd, WrappingSub, Zero
};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    ExecutionError
};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub, SubAssign};

macro_rules! impl_casper_type_numeric_wrapper {
    ( $( $ty:ident ),+ ) => {
        $(
            /// Little-endian large integer type
            #[derive(Debug, Default, PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
            pub struct $ty {
                inner: casper_types::$ty
            }

            impl $ty {
                pub fn as_u32(&self) -> u32 {
                    self.inner.as_u32()
                }

                pub fn max_value() -> Self {
                    Self {
                        inner: casper_types::$ty::MAX
                    }
                }

                pub fn inner(&self) -> casper_types::$ty {
                    self.inner
                }

                pub fn zero() -> Self {
                    Zero::zero()
                }

                pub fn one() -> Self {
                    One::one()
                }

                pub fn is_zero(&self) -> bool {
                    <Self as Zero>::is_zero(self)
                }

                pub fn checked_add(&self, v: $ty) -> Option<$ty> {
                    casper_types::$ty::checked_add(self.inner, v.inner).map(|inner| $ty { inner })
                }

                pub fn checked_sub(&self, v: $ty) -> Option<$ty> {
                    casper_types::$ty::checked_sub(self.inner, v.inner).map(|inner| $ty { inner })
                }

                pub fn checked_mul(&self, v: $ty) -> Option<$ty> {
                    casper_types::$ty::checked_mul(self.inner, v.inner).map(|inner| $ty { inner })
                }

                pub fn checked_div(&self, v: $ty) -> Option<$ty> {
                    casper_types::$ty::checked_div(self.inner, v.inner).map(|inner| $ty { inner })
                }

                pub fn overflowing_add(&self, v: $ty) -> ($ty, bool) {
                    let (inner, is_overflowed) = casper_types::$ty::overflowing_add(self.inner, v.inner);
                    ($ty { inner }, is_overflowed)
                }

                pub fn overflowing_sub(&self, v: $ty) -> ($ty, bool) {
                    let (inner, is_overflowed) = casper_types::$ty::overflowing_sub(self.inner, v.inner);
                    ($ty { inner }, is_overflowed)
                }

                pub fn abs_diff(&self, v: $ty) -> $ty {
                    Self {
                        inner: casper_types::$ty::abs_diff(self.inner, v.inner)
                    }
                }

                pub fn saturating_mul(&self, v: $ty) -> $ty {
                    Self {
                        inner: casper_types::$ty::saturating_mul(self.inner, v.inner)
                    }
                }

                pub fn saturating_sub(&self, v: $ty) -> $ty {
                    Self {
                        inner: casper_types::$ty::saturating_sub(self.inner, v.inner)
                    }
                }

                pub fn from_dec_str(s: &str) -> Result<Self, String> {
                    Self::from_str_radix(s, 10)
                }
            }

            // Trait implementations for unifying U* as numeric types
            impl Zero for $ty {
                fn zero() -> Self {
                    Self {
                        inner: casper_types::$ty::zero()
                    }
                }

                fn is_zero(&self) -> bool {
                    self.inner.is_zero()
                }
            }

            impl One for $ty {
                fn one() -> Self {
                    Self {
                        inner: casper_types::$ty::one()
                    }
                }
            }

            // Requires Zero and One to be implemented
            impl Num for $ty {
                type FromStrRadixErr = String;
                fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                    if radix == 10 {
                       casper_types::$ty::from_dec_str(str)
                            .map(|inner| Self { inner })
                            .map_err(|_| String::from("FromDecStr"))
                    } else {
                        Err(String::from("InvalidRadix"))
                    }
                }
            }

            // Requires Num to be implemented
            impl Unsigned for $ty {}

            // Additional numeric trait, which also holds for these types
            impl Bounded for $ty {
                fn min_value() -> Self {
                    $ty::zero()
                }

                fn max_value() -> Self {
                    Self { inner: casper_types::$ty::MAX }
                }
            }

            // Instead of implementing arbitrary methods we can use existing traits from num_trait
            // crate.
            impl WrappingAdd for $ty {
                fn wrapping_add(&self, other: &$ty) -> $ty {
                    Self { inner: self.inner.overflowing_add(other.inner).0 }
                }
            }

            impl WrappingSub for $ty {
                fn wrapping_sub(&self, other: &$ty) -> $ty {
                    Self { inner: self.inner.overflowing_sub(other.inner).0 }
                }
            }

            impl CheckedMul for $ty {
                fn checked_mul(&self, v: &$ty) -> Option<$ty> {
                    casper_types::$ty::checked_mul(self.inner, v.inner).map(|inner| $ty { inner })
                }
            }

            impl CheckedSub for $ty {
                fn checked_sub(&self, v: &$ty) -> Option<$ty> {
                    casper_types::$ty::checked_sub(self.inner, v.inner).map(|inner| $ty { inner })
                }
            }

            impl FromBytes for $ty {
                fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
                    casper_types::$ty::from_bytes(bytes).map(|(inner, bytes)| ($ty {inner}, bytes))
                }
            }

            impl ToBytes for $ty {
                fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
                    self.inner.to_bytes()
                }

                fn serialized_length(&self) -> usize {
                    self.inner.serialized_length()
                }
            }

            impl CLTyped for $ty {
                fn cl_type() -> casper_types::CLType {
                    casper_types::$ty::cl_type()
                }
            }

            impl OverflowingAdd for $ty {
                fn overflowing_add(self, rhs: Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.inner.overflowing_add(rhs.inner);
                    match is_overflowed {
                        true => Err(ArithmeticsError::AdditionOverflow.into()),
                        false => Ok(Self { inner: res })
                    }
                }
            }

            impl OverflowingSub for $ty {
                fn overflowing_sub(self, rhs: Self) -> Result<Self, ExecutionError> {
                    let (res, is_overflowed)  = self.inner.overflowing_sub(rhs.inner);
                    match is_overflowed {
                        true => Err(ArithmeticsError::SubtractingOverflow.into()),
                        false => Ok(Self { inner: res })
                    }
                }
            }

            impl<T> From<T> for $ty where T: Into<casper_types::$ty> {
                fn from(value: T) -> Self {
                    Self {
                        inner: value.into()
                    }
                }
            }

            impl Add for $ty {
                type Output = Self;

                fn add(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner + rhs.inner,
                    }
                }
            }

            impl AddAssign for $ty {
                fn add_assign(&mut self, rhs: Self) {
                    self.inner += rhs.inner;
                }
            }

            impl Sub for $ty {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner - rhs.inner,
                    }
                }
            }

            impl SubAssign for $ty {
                fn sub_assign(&mut self, rhs: Self) {
                    self.inner -= rhs.inner;
                }
            }

            impl Add<u32> for $ty {
                type Output = Self;

                fn add(self, rhs: u32) -> Self::Output {
                    Self {
                        inner: self.inner + rhs,
                    }
                }
            }

            impl Sub<u32> for $ty {
                type Output = Self;

                fn sub(self, rhs: u32) -> Self::Output {
                    Self {
                        inner: self.inner - rhs,
                    }
                }
            }

            impl Mul for $ty {
                type Output = Self;

                fn mul(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner * rhs.inner,
                    }
                }
            }

            impl Rem for $ty {
                type Output = Self;

                fn rem(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner % rhs.inner,
                    }
                }
            }

            impl Div for $ty {
                type Output = Self;

                fn div(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner / rhs.inner,
                    }
                }
            }

            impl std::fmt::Display for $ty {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(&self.inner, f)
                }
            }

        )+
    };
}

impl_casper_type_numeric_wrapper!(U128, U256, U512);

impl U512 {
    pub fn to_u256(self) -> Result<U256, ArithmeticsError> {
        let inner = self.inner();
        if exceeds_u256(inner) {
            return Err(ArithmeticsError::AdditionOverflow);
        }

        let src = inner.0;
        let mut words = [0u64; 4];
        words.copy_from_slice(&src[0..4]);

        Ok(U256 {
            inner: casper_types::U256(words)
        })
    }
}

impl U256 {
    pub fn to_u512(self) -> Result<U512, ArithmeticsError> {
        let inner = self.inner();
        let mut bytes = [0u8; 32];
        inner.to_little_endian(&mut bytes);
        let balance = casper_types::U512::from_little_endian(&bytes);
        Ok(U512 { inner: balance })
    }

    pub fn to_balance(self) -> Result<U512, ArithmeticsError> {
        self.to_u512()
    }
}

impl Hash for U256 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().0.hash(state);
    }
}

fn exceeds_u256(value: casper_types::U512) -> bool {
    let max_u256 = (casper_types::U512::one() << 256) - 1;
    value > max_u256
}

impl From<U512> for u32 {
    fn from(value: U512) -> Self {
        value.inner().0[0] as u32
    }
}

#[cfg(test)]
mod tests {
    use casper_types::bytesrepr::{FromBytes, ToBytes};
    use odra_types::arithmetic::ArithmeticsError;

    use crate::{U256, U512};

    #[test]
    fn test_u512_to_256() {
        let value = U512::from(100);
        assert_eq!(value.to_u256().unwrap(), U256::from(100));

        let value = U256::max_value();
        let bytes = value.to_bytes().unwrap();
        let (value, remainder) = U512::from_bytes(&bytes).unwrap();
        assert!(remainder.is_empty());
        assert_eq!(value.to_u256().unwrap(), U256::max_value());

        let more_then_max = value + U512::one();
        assert_eq!(
            more_then_max.to_u256().unwrap_err(),
            ArithmeticsError::AdditionOverflow
        );
    }

    #[test]
    fn test_u256_to_512() {
        let value = U256::one();
        assert_eq!(value.to_u512().unwrap(), U512::one());

        let value = U256::max_value();
        let max_u256 = (casper_types::U512::one() << 256) - 1;
        let max_u256 = U512::from(max_u256);
        assert_eq!(value.to_u512().unwrap(), max_u256);
    }
}
