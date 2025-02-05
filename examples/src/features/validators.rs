//! This example shows how to test a contract.

use odra::{
    casper_types::{PublicKey, U512},
    prelude::*
};

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

    /// Stake the amount of tokens
    #[odra(payable)]
    pub fn stake(&mut self) {
        let amount = self.env().attached_value();
        if amount.is_zero() {
            self.env().revert(ValError::InsufficientBalance);
        }
        self.env().delegate(self.validator.get().unwrap(), amount);
    }

    /// Undelegate the amount from the validator
    pub fn unstake(&mut self, amount: U512) {
        self.env().undelegate(self.validator.get().unwrap(), amount);
    }

    /// Withdraw the amount from the validator
    pub fn withdraw(&mut self, amount: U512) {
        self.env().transfer_tokens(&self.env().caller(), &amount);
    }

    /// Get the currently delegated amount
    pub fn currently_delegated_amount(&self) -> U512 {
        self.env().delegated_amount(self.validator.get().unwrap())
    }

    /// Get the current casper balance of the contract
    pub fn current_casper_balance(&self) -> U512 {
        self.env().self_balance()
    }
}

/// Error enum for the ValidatorsContract
#[odra::odra_error]
pub enum ValError {
    /// Error for insufficient balance
    InsufficientBalance = 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validators() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::casper_types::U512;
        use odra::host::Deployer;
        use odra::host::HostRef;

        let test_env = odra_test::env();
        let auction_delay = test_env.auction_delay();
        // Should we put it into host method?
        let unbonding_delay = auction_delay * 8;

        test_env.set_caller(test_env.get_account(0));
        let mut staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: test_env.get_validator(0)
            }
        );

        // Setup validators in vm
        let inital_account_balance = test_env.balance_of(&test_env.get_account(0));

        // Stake some amount
        let staking_amount = U512::from(1_000_000_000_000u64);
        staking.with_tokens(staking_amount).stake();
        assert_eq!(staking.currently_delegated_amount(), staking_amount);
        assert_eq!(
            test_env.balance_of(&test_env.get_account(0)),
            inital_account_balance - staking_amount
        );

        // Advance time, run auctions and give off rewards
        // TODO: Why it works with after 6 and not with auction delay? 0th validator
        // should be the first to get rewards...
        test_env.advance_with_auctions(auction_delay * 6);

        // Check that the amount is greater than the staking amount
        let staking_with_reward = staking.currently_delegated_amount();
        assert!(staking_with_reward > staking_amount);

        // Unstake
        staking.unstake(staking_with_reward);
        assert_eq!(staking.currently_delegated_amount(), U512::from(0));

        // Withdraw should first fail, as we need to wait for auction delay
        staking.try_withdraw(staking_with_reward).unwrap_err();
        // To confirm, the contract balance should be 0
        assert_eq!(staking.current_casper_balance(), U512::from(0));

        // Advance time, run auctions and give off rewards
        test_env.advance_with_auctions(unbonding_delay);
        assert_eq!(staking.current_casper_balance(), staking_with_reward);
        staking.withdraw(staking_with_reward);
        assert_eq!(staking.current_casper_balance(), U512::from(0));

        // The user now should have the tokens
        assert_eq!(
            test_env.balance_of(&test_env.get_account(0)),
            staking_with_reward + inital_account_balance - staking_amount
        );
    }
}
