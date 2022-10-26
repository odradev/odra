use odra::types::{Address, U256};

#[odra::module]
pub struct BalanceChecker {}

#[odra::module]
impl BalanceChecker {
    pub fn check_balance(&self, token: Address, account: Address) -> U256 {
        TokenRef::at(token).balance_of(account)
    }
}

#[odra::external_contract]
trait Token {
    fn balance_of(&self, address: Address) -> U256;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erc20;
    use odra::TestEnv;

    #[test]
    fn balance_checker() {
        let (owner, second_account) = (TestEnv::get_account(0), TestEnv::get_account(1));
        let balance_checker = BalanceChecker::deploy();
        let token = erc20::tests::setup();
        let expected_owner_balance = erc20::tests::INITIAL_SUPPLY;

        // Owner of the token should have positive balance.
        let balance = balance_checker.check_balance(token.address(), owner);
        assert_eq!(balance.as_u32(), expected_owner_balance);

        // Different account should have zero balance.
        let balance = balance_checker.check_balance(token.address(), second_account);
        assert!(balance.is_zero());
    }
}
