use crate::{CLTyped, FromBytes, ToBytes};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    ExecutionError
};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub};

macro_rules! impl_casper_type_numeric_wrapper {
    ( $( $ty:ident ),+ ) => {
        $(
            /// Little-endian large integer type
            #[derive(Debug, Default, PartialEq, Eq, PartialOrd, Clone, Copy)]
            pub struct $ty {
                inner: casper_types::$ty
            }

            impl $ty {
                pub fn zero() -> Self {
                    Self {
                        inner: casper_types::$ty::zero()
                    }
                }

                pub fn one() -> Self {
                    Self {
                        inner: casper_types::$ty::one()
                    }
                }

                pub fn inner(&self) -> casper_types::$ty {
                    self.inner
                }

                pub fn is_zero(&self) -> bool {
                    self.inner.is_zero()
                }

                pub fn as_u32(&self) -> u32 {
                    self.inner.as_u32()
                }

                pub fn max() -> Self {
                    Self {
                        inner: casper_types::$ty::MAX
                    }
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

            impl Sub for $ty {
                type Output = Self;

                fn sub(self, rhs: Self) -> Self::Output {
                    Self {
                        inner: self.inner - rhs.inner,
                    }
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

#[cfg(test)]
mod tests {
    use casper_types::bytesrepr::{FromBytes, ToBytes};
    use odra_types::arithmetic::ArithmeticsError;

    use crate::{U256, U512};

    #[test]
    fn test_u512_to_256() {
        let value = U512::from(100);
        assert_eq!(value.to_u256().unwrap(), U256::from(100));

        let value = U256::max();
        let bytes = value.to_bytes().unwrap();
        let (value, remainder) = U512::from_bytes(&bytes).unwrap();
        assert!(remainder.is_empty());
        assert_eq!(value.to_u256().unwrap(), U256::max());

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

        let value = U256::max();
        let max_u256 = (casper_types::U512::one() << 256) - 1;
        let max_u256 = U512::from(max_u256);
        assert_eq!(value.to_u512().unwrap(), max_u256);
    }
}
