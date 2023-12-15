use odra::{Address, U256};
use odra::prelude::*;

#[odra::module]
pub struct BalanceChecker {}

#[odra::module]
impl BalanceChecker {
    pub fn check_balance(&self, token: &Address, account: &Address) -> U256 {
        TokenHostRef::at(token).balance_of(account)
    }
}

#[odra::external_contract]
pub trait Token {
    fn balance_of(&self, owner: &Address) -> U256;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_checker() {
        let env = odra::test_env();
        let (owner, second_account) = (env.get_account(0), env.get_account(1));
        let balance_checker = BalanceCheckerDeployer::init(&env);
        let token = tests::setup();
        let expected_owner_balance = tests::INITIAL_SUPPLY;

        // Owner of the token should have positive balance.
        let balance = balance_checker.check_balance(token.address(), &owner);
        assert_eq!(balance.as_u32(), expected_owner_balance);

        // Different account should have zero balance.
        let balance = balance_checker.check_balance(token.address(), &second_account);
        assert!(balance.is_zero());
    }

    #[test]
    fn is_module() {
        assert!(BalanceChecker::is_module());
        assert!(!U256::is_module());
    }
}
