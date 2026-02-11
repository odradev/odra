//! Module containing DogContract. It is used in docs to explain how to interact with the storage.
use odra::{casper_types::U256, prelude::*};

/// A simple contract that represents a dog.
#[odra::module]
pub struct MockDex;

#[odra::module]
impl MockDex {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self) {}

    pub fn get_reserves(&self) -> (U256, U256) {
        (U256::zero(), U256::zero())
    }

    pub fn swap(&mut self, amount_in: U256) -> U256 {
        amount_in * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn init_test() {
        let test_env = odra_test::env();
        let mut contract = MockDex::deploy(&test_env, NoArgs);

        assert_eq!(contract.try_swap_no_ret(1.into()), Ok(()));
        assert_eq!(contract.try_swap(1.into()), Ok(2.into()));
    }
}
