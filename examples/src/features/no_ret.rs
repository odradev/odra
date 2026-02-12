//! Module containing a mock dex contract. It is used to demonstrate the no-ret feature.
//!
//! The no-ret feature allows you to generate a version of a mutable function
//! that ignores the result of the call.
//!
//! If there is a module defined as:
//!
//! ```ignore
//! #[odra::module]
//! pub struct MockDex;
//!
//! #[odra::module]
//! impl MockDex {
//!     pub fn swap(&mut self, amount_in: U256) -> U256 {
//!         amount_in * 2
//!     }
//! }
//! ```
//!
//! The generated code will contain an extra impl block with `_no_ret`-suffixed functions.
//!
//! ```ignore
//! impl MockDexHostRef {
//!     /// Ignores the result of the call.
//!     pub fn swap_no_ret(&mut self, amount_in: U256) {
//!         self.try_swap_no_ret(amount_in).unwrap()
//!     }
//!
//!     /// Ignores the result of the call.
//!     pub fn try_swap_no_ret(&mut self, amount_in: U256) -> OdraResult<()> {
//!         let _ = self.try_swap(amount_in)?;
//!         Ok(())
//!     }
//! }
//! ```
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
