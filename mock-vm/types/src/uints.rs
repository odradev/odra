use borsh::{BorshDeserialize, BorshSerialize};
use odra_types::{
    arithmetic::{ArithmeticsError, OverflowingAdd, OverflowingSub},
    impl_overflowing_add_sub, ExecutionError
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

impl_overflowing_add_sub!(U256, U512);
