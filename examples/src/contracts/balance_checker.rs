//! Example of Balance Checker contract.
use odra::prelude::*;
use odra::{casper_types::U256, Address};

/// BalanceChecker contract.
#[odra::module]
pub struct BalanceChecker;

#[odra::module]
impl BalanceChecker {
    /// Checks the balance of the given account for the given token.
    pub fn check_balance(&self, token: &Address, account: &Address) -> U256 {
        TokenContractRef::new(self.env(), *token).balance_of(*account)
    }
}

/// Token contract interface.
#[odra::external_contract]
pub trait Token {
    /// Returns the balance of the given account.
    fn balance_of(&self, owner: &Address) -> U256;
}

#[cfg(test)]
mod tests {
    use odra::host::{Deployer, HostRef, NoArgs};

    use super::*;
    use crate::contracts::owned_token::tests::{setup, INITIAL_SUPPLY};

    #[test]
    fn balance_checker() {
        let token = setup();
        let env = token.env().clone();
        let (owner, second_account) = (env.get_account(0), env.get_account(1));
        let balance_checker = BalanceCheckerHostRef::deploy(&env, NoArgs);
        let expected_owner_balance = INITIAL_SUPPLY;

        // Owner of the token should have positive balance.
        let balance = balance_checker.check_balance(*token.address(), owner);
        assert_eq!(balance.as_u32(), expected_owner_balance);

        // Different account should have zero balance.
        let balance = balance_checker.check_balance(*token.address(), second_account);
        assert!(balance.is_zero());
    }
}
