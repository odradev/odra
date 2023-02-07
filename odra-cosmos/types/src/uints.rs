use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    impl_overflowing_add_sub, ExecutionError
};
use serde::{Deserialize, Serialize};
use uint::construct_uint;

use crate::Typed;

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

// TODO: Do it better
impl Into<String> for U256 {
    fn into(self) -> String {
        String::from_utf8(self.as_u128().to_le_bytes().to_vec()).unwrap()
    }
}

// TODO: Do it better
impl Into<String> for U512 {
    fn into(self) -> String {
        String::from_utf8(self.as_u128().to_le_bytes().to_vec()).unwrap()
    }
}


impl_overflowing_add_sub!(U256, U512);
