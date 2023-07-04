use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::{
    Bounded, CheckedMul, CheckedSub, Num, One, Unsigned, WrappingAdd, WrappingSub, Zero
};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    impl_overflowing_add_sub, ExecutionError
};
use uint::construct_uint;

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U128(2);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U256(4);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U512(8);
}

impl From<U256> for U512 {
    fn from(val: U256) -> Self {
        val.to_u512()
            .unwrap_or_else(|_| panic!("U256 to U512 conversion failed"))
    }
}

impl U256 {
    pub fn to_u256(self) -> Result<U256, ArithmeticsError> {
        Ok(self)
    }

    pub fn to_u512(self) -> Result<U512, ArithmeticsError> {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        Ok(U512::from_little_endian(&bytes))
    }

    pub fn to_balance(self) -> Result<U256, ArithmeticsError> {
        Ok(self)
    }
}

impl_overflowing_add_sub!(U128, U256, U512);

macro_rules! impl_traits_for_uint {
    ($type:ident, $total_bytes:expr) => {
        impl Zero for $type {
            fn zero() -> Self {
                $type::zero()
            }

            fn is_zero(&self) -> bool {
                self.is_zero()
            }
        }

        impl One for $type {
            fn one() -> Self {
                $type::one()
            }
        }

        // Requires Zero and One to be implemented
        impl Num for $type {
            type FromStrRadixErr = String;
            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                if radix == 10 {
                    $type::from_dec_str(str).map_err(|_| String::from("FromDecStr"))
                } else {
                    Err(String::from("InvalidRadix"))
                }
            }
        }

        // Requires Num to be implemented
        impl Unsigned for $type {}

        // Additional numeric trait, which also holds for these types
        impl Bounded for $type {
            fn min_value() -> Self {
                $type::zero()
            }

            fn max_value() -> Self {
                $type::MAX
            }
        }

        // Instead of implementing arbitrary methods we can use existing traits from num_trait
        // crate.
        impl WrappingAdd for $type {
            fn wrapping_add(&self, other: &$type) -> $type {
                self.overflowing_add(*other).0
            }
        }

        impl WrappingSub for $type {
            fn wrapping_sub(&self, other: &$type) -> $type {
                self.overflowing_sub(*other).0
            }
        }

        impl CheckedMul for $type {
            fn checked_mul(&self, v: &$type) -> Option<$type> {
                $type::checked_mul(*self, *v)
            }
        }

        impl CheckedSub for $type {
            fn checked_sub(&self, v: &$type) -> Option<$type> {
                $type::checked_sub(*self, *v)
            }
        }
    };
}

impl_traits_for_uint!(U128, 16);
impl_traits_for_uint!(U256, 32);
impl_traits_for_uint!(U512, 64);
