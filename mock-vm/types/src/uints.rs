use borsh::{BorshDeserialize, BorshSerialize};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    impl_overflowing_add, impl_overflowing_sub, ExecutionError,
};
use uint::construct_uint;

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U256(4);
}

construct_uint! {
    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct U512(8);
}

impl_overflowing_add!(U256, U512);
impl_overflowing_sub!(U256, U512);
