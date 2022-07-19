use casper_types::{U256, U512, U128};

pub trait OverflowingAdd: Sized {
    fn overflowing_add(self, rhs: Self) -> (Self, bool);
}

pub trait OverflowingSub: Sized {
    fn overflowing_sub(self, rhs: Self) -> (Self, bool);
}

macro_rules! impl_overflowing_add {
    ( $( $ty:ty ),+ ) => {
        $( impl OverflowingAdd for $ty {
            fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                self.overflowing_add(rhs)
            }
        })+
    };
}

macro_rules! impl_overflowing_sub {
    ( $( $ty:ty ),+ ) => {
        $( impl OverflowingSub for $ty {
            fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                self.overflowing_sub(rhs)
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
