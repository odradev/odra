//! This example shows how to test delegation.

use odra::{
    casper_types::{PublicKey, U512},
    prelude::*
};

/// Contract presenting the test
#[odra::module]
pub struct ValidatorsContract {
    validator: Var<PublicKey>
}

/// Implementation of the TestingContract
#[odra::module]
impl ValidatorsContract {
    /// Initializes the contract with the validator's public key
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

    /// Get minimum delegation amount
    pub fn get_minimum_delegation_amount(&self) -> u64 {
        self.env()
            .get_validator_info(self.validator.get().unwrap())
            .unwrap()
            .minimum_delegation_amount
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
    use alloc::vec::Vec;
    use odra::casper_types::U512;
    use odra::host::HostRef;

    #[test]
    fn test_advance_with_auctions() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::host::Deployer;
        use odra::host::HostRef;

        let test_env = odra_test::env();
        let auction_delay = test_env.auction_delay();

        // Deploy 5 staking contracts, one for each validator
        let mut staking_contracts = Vec::new();
        let staking_amount = U512::from(1_000_000_000_000u64);
        for i in 0..5 {
            test_env.set_caller(test_env.get_account(i));
            let staking = ValidatorsContract::deploy(
                &test_env,
                ValidatorsContractInitArgs {
                    validator: test_env.get_validator(i)
                }
            );
            staking.with_tokens(staking_amount).stake();
            assert_eq!(staking.currently_delegated_amount(), staking_amount);
            staking_contracts.push(staking);
        }

        // First auction for dwarves, we are waiting for the auction delay to pass
        test_env.advance_with_auctions(auction_delay);

        // Advance one auction at a time and verify rewards
        test_env.advance_with_auctions(auction_delay);

        // Now we should have rewards for each validator
        for i in 0..5 {
            let contract = staking_contracts.get(i).unwrap();
            assert!(
                contract.currently_delegated_amount() > staking_amount,
                "Validator should have received rewards in contract no {}, delegated amount: {}, staking amount: {}",
                i, contract.currently_delegated_amount(), staking_amount
            );
        }
    }

    #[test]
    fn test_validators() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::casper_types::U512;
        use odra::host::Deployer;
        use odra::host::HostRef;

        let test_env = odra_test::env();
        let auction_delay = test_env.auction_delay();
        let unbonding_delay = test_env.unbonding_delay();

        test_env.set_caller(test_env.get_account(0));
        let mut staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: test_env.get_validator(0)
            }
        );

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
        test_env.advance_with_auctions(auction_delay * 2);

        // Check that the amount is greater than the staking amount
        let staking_with_reward = staking.currently_delegated_amount();
        assert!(staking_with_reward > staking_amount);

        let reward = staking_with_reward - staking_amount;

        // Unstake
        staking.unstake(staking_with_reward);
        assert_eq!(staking.currently_delegated_amount(), U512::from(0));

        // Withdraw should first fail, as we need to wait for auction delay
        staking.try_withdraw(staking_with_reward).unwrap_err();
        // To confirm, the contract balance should be 0
        assert_eq!(staking.current_casper_balance(), U512::from(0));

        // Advance time, run auctions and give off rewards
        test_env.advance_with_auctions(unbonding_delay * 2);

        staking.withdraw(staking_with_reward);

        // The user now should have the tokens
        assert_eq!(
            test_env.balance_of(&test_env.get_account(0)),
            inital_account_balance + reward
        );
    }
    #[test]
    fn test_config() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::casper_types::U512;
        use odra::host::Deployer;
        use odra::host::HostRef;
        use odra::prelude::Addressable;

        let test_env = odra_test::env();
        let validator = test_env.get_validator(0);

        test_env.set_caller(test_env.get_account(0));
        let staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: validator.clone()
            }
        );

        let staking_amount = U512::from(1_000_000_000_000u64);
        staking.with_tokens(staking_amount).stake();

        // HostEnv's staked amount should be equal to the staking amount
        assert_eq!(
            test_env.delegated_amount(staking.address(), validator),
            staking_amount
        );

        // And to the amount reported by the contract
        assert_eq!(staking.currently_delegated_amount(), staking_amount);
    }

    #[test]
    fn test_remove_validator() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::casper_types::U512;
        use odra::host::Deployer;
        use odra::host::HostRef;
        use odra::prelude::Addressable;

        let test_env = odra_test::env();
        let unbonding_delay = test_env.unbonding_delay();

        test_env.set_caller(test_env.get_account(0));
        let staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: test_env.get_validator(0)
            }
        );

        // Stake some amount
        let staking_amount = U512::from(1_000_000_000_000u64);
        staking.with_tokens(staking_amount).stake();
        assert_eq!(staking.currently_delegated_amount(), staking_amount);

        // Remove the validator
        test_env.remove_validator(0);

        assert_eq!(staking.currently_delegated_amount(), U512::zero());
        assert_eq!(test_env.balance_of(&staking.address()), U512::zero());

        // Advance time, run auctions and give off rewards
        test_env.advance_with_auctions(unbonding_delay * 2);

        // No rewards should be given, as the validator was removed,
        // but the cspr should be returned
        assert_eq!(staking.currently_delegated_amount(), U512::zero());
        assert_eq!(test_env.balance_of(&staking.address()), staking_amount);
    }

    #[test]
    fn test_validator_info() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::host::Deployer;
        let test_env = odra_test::env();
        let validator = test_env.get_validator(0);

        test_env.set_caller(test_env.get_account(0));
        let staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: validator.clone()
            }
        );

        let minimum_delegation_amount = staking.get_minimum_delegation_amount();

        assert_eq!(minimum_delegation_amount, 500_000_000_000u64);
    }

    #[test]
    fn test_delegation() {
        use crate::features::validators::{ValidatorsContract, ValidatorsContractInitArgs};
        use odra::host::Deployer;
        let test_env = odra_test::env();
        let validator = test_env.get_validator(0);

        test_env.set_caller(test_env.get_account(0));
        let mut staking = ValidatorsContract::deploy(
            &test_env,
            ValidatorsContractInitArgs {
                validator: validator.clone()
            }
        );

        let minimum_delegation_amount = staking.get_minimum_delegation_amount().into();

        staking.with_tokens(minimum_delegation_amount).stake();

        assert_eq!(staking.currently_delegated_amount(), minimum_delegation_amount);

        test_env.advance_with_auctions(test_env.auction_delay() * 2);

        assert_eq!(staking.currently_delegated_amount(), U512::from(500_000_099_998u64));

        staking.unstake(U512::from(500_000_000_000u64));

        test_env.advance_with_auctions(test_env.auction_delay());
        test_env.advance_with_auctions(test_env.unbonding_delay());

        assert_eq!(staking.currently_delegated_amount(), U512::zero());
    }
}
