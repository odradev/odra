use casper_types::{U256, U512};

use crate::arithmetic::ArithmeticsError;

pub trait ToU256 {
    fn to_u256(self) -> Result<U256, ArithmeticsError>;
}

pub trait ToU512 {
    fn to_u512(self) -> U512;
}

impl ToU256 for U512 {
    fn to_u256(self) -> Result<U256, ArithmeticsError> {
        if exceeds_u256(self) {
            return Err(ArithmeticsError::AdditionOverflow);
        }

        let src = self.0;
        let mut words = [0u64; 4];
        words.copy_from_slice(&src[0..4]);

        Ok(U256(words))
    }
}

impl ToU512 for U256 {
    fn to_u512(self) -> U512 {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        U512::from_little_endian(&bytes)
    }
}

fn exceeds_u256(value: U512) -> bool {
    let max_u256 = (U512::one() << 256) - 1;
    value > max_u256
}
