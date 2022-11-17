use crate::{CLTyped, FromBytes, ToBytes};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    ExecutionError
};
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
        )+
    };
}

impl_casper_type_numeric_wrapper!(U256, U512);
