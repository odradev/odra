//! This example shows how to test a contract.

use core::fmt::Error;

use odra::casper_types::{PublicKey, U512};
use odra::prelude::*;

/// Contract presenting the testing abilities of the Odra Framework
#[odra::module]
pub struct ValidatorsContract {
    validator: Var<PublicKey>
}

/// Implementation of the TestingContract
#[odra::module]
impl ValidatorsContract {
    /// Initializes the contract with the name
    pub fn init(&mut self, validator: PublicKey) {
        self.validator.set(validator);
    }

    #[odra(payable)]
    pub fn topup(&mut self) {

    }

    pub fn stake(&mut self) {
        let amount = self.env().self_balance();
        if amount.is_zero() {
            self.env().revert(ValError::InsufficientBalancee);
        }
        self.env().delegate(self.validator.get().unwrap(), amount);
    }

    pub fn unstake(&mut self, amount: U512) {
        self.env().undelegate(self.validator.get().unwrap(), amount);
    }

    pub fn currently_delegated_amount(&self) -> U512 {
        self.env().delegated_amount(self.validator.get().unwrap())
    }

    pub fn current_casper_balance(&self) -> U512 {
        self.env().self_balance()
    }
}

#[odra::odra_error]
pub enum ValError {
    InsufficientBalancee = 1,
}

#[cfg(test)]
mod tests {
    use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
    use odra::casper_types::{PublicKey, U512};
    use odra::host::{HostRef, NoArgs};
    use odra::{host::Deployer, prelude::*};

    /// Time in milliseconds for one era. On livenet it's 120 minutes. On local, by default it's 41 seconds.
    pub const ERA_DURATION: u64 = 41 * 1000;

    #[test]
    fn test_validators() {
        let test_env = odra_test::env();
        test_env.set_caller(test_env.get_account(0));
        let mut staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: test_env.get_validator()
            }
        );

        // Setup validators in vm
        let validator1: PublicKey = test_env.get_validator();

        // Stake some amount
        let staking_amount = U512::from(1_000_000_000_000u64);
        staking.with_tokens(staking_amount).topup();
        staking.stake();
        assert_eq!(staking.currently_delegated_amount(), staking_amount);
        panic!("dupppa");
        test_env.advance_block_time(ERA_DURATION * 10);
        
        let balance = staking.current_casper_balance();
        assert_eq!(balance, staking_amount/2);
        // assert!(staking.currently_delegated_amount() > staking_amount);
        staking.unstake(staking_amount);
        
        test_env.advance_block_time(ERA_DURATION * 10);
        
        assert!(staking.current_casper_balance() > staking_amount);
        assert_eq!(staking.currently_delegated_amount(), U512::zero());
    }
}
