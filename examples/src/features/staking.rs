//! This example shows how to handle signature verification in a contract.
use odra::casper_types::PublicKey;
use odra::prelude::*;

/// A Contract that verifies signatures.
#[odra::module]
pub struct DelegateContract;

#[odra::module]
impl DelegateContract {
    /// Stakes the user's tokens.
    #[odra(payable)]
    pub fn delegate(&self, validator: PublicKey) {
        let amount = self.env().attached_value();
        self.env().delegate(validator, amount);
    }

    /// Undelegates the user's tokens.
    /// After unbonding time, the tokens will be transferred to the contract.
    pub fn undelegate(&self, validator: PublicKey) {
        let delegated_amount = self.env().delegated_amount(validator.clone());
        self.env().undelegate(validator, delegated_amount);
    }

    /// Withdraws the tokens to the caller's account.
    pub fn withdraw(&self) {
        let amount = self.env().self_balance();
        self.env().transfer_tokens(&self.env().caller(), &amount)
    }
}

#[cfg(test)]
mod test {
    use crate::features::staking::DelegateContract;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn delegate_works() {
        let test_env = odra_test::env();
        let delegate_contract = DelegateContract::deploy(&test_env, NoArgs);
    }
}
