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

impl_overflowing_add_sub!(U256, U512);
