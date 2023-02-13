use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    impl_overflowing_add_sub, ExecutionError
};
use serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}

construct_uint! {
    #[derive(Serialize, Deserialize)]
    pub struct U512(8);
}

impl U256 {
    pub fn to_u256(self) -> Result<U256, ArithmeticsError> {
        Ok(self)
    }

    pub fn to_balance(self) -> Result<U256, ArithmeticsError> {
        Ok(self)
    }
}

macro_rules! impl_into_string {
    ($name:ident, $n_words:tt) => {
        impl Into<String> for $name {
            fn into(self) -> String {
                let mut buf = [0_u8; $n_words * 20];
                let mut i = buf.len() - 1;
                let mut current = self;
                let ten = $name::from(10);

                loop {
                    let digit = (current % ten).low_u64() as u8;
                    buf[i] = digit + b'0';
                    current = current / ten;
                    if current.is_zero() {
                        break;
                    }
                    i -= 1;
                }

                // sequence of `'0'..'9'` chars is guaranteed to be a valid UTF8 string
                unsafe { ::core::str::from_utf8_unchecked(&buf[i..]) }.to_string()
            }
        }
    };
}

impl_overflowing_add_sub!(U256, U512);
impl_into_string!(U256, 4);
impl_into_string!(U512, 8);

#[cfg(test)]
mod test {
    use crate::{U256, U512};

    #[test]
    fn to_str() {
        let expected_max_u512 = String::from("13407807929942597099574024998205846127479365820592393377723561443721764030073546976801874298166903427690031858186486050853753882811946569946433649006084095");
        let expected_max_u256 = String::from(
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        );
        let actual_max_u256: String = U256::MAX.into();
        let actual_max_u512: String = U512::MAX.into();

        assert_eq!(expected_max_u512, actual_max_u512);
        assert_eq!(expected_max_u256, actual_max_u256);
    }
}
