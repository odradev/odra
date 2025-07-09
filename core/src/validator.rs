//! Validator related types.
use casper_types::U512;

/// ValidatorInfo contains information about a validator.
#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    /// The amount of tokens staked.
    pub staked_amount: U512,
    /// The minimum amount of tokens that must be delegated to the validator.
    pub minimum_delegation_amount: u64
}

impl ValidatorInfo {
    /// Creates a new ValidatorInfo with the given staked amount and minimum delegation amount.
    pub fn new(staked_amount: U512, minimum_delegation_amount: u64) -> Self {
        ValidatorInfo {
            staked_amount,
            minimum_delegation_amount
        }
    }

    /// Sets the ValidatorInfo with the staked amount set to the given value.
    pub fn set_staked_amount(&mut self, staked_amount: U512) {
        self.staked_amount = staked_amount;
    }

    /// Sets the ValidatorInfo with the minimum delegation amount set to the given value.
    pub fn set_minimum_delegation_amount(&mut self, minimum_delegation_amount: u64) {
        self.minimum_delegation_amount = minimum_delegation_amount;
    }
}
