use odra::{
    types::{Address, U256},
    Instance, Mapping, UnwrapOrRevert, Variable
};

use crate::erc20::Erc20;

#[odra::module]
pub struct TokenManager {
    tokens: Mapping<String, Erc20>,
    count: Variable<u32>
}

#[odra::module]
impl TokenManager {
    pub fn add_token(&mut self, token_name: String, decimals: u8, symbol: String, name: String) {
        let count = self.count.get_or_default();
        let mut module = Erc20::instance(&count.to_string());
        module.init(name, symbol, decimals, U256::from(0));

        self.tokens.set(&token_name, module);
        self.count.set(count + 1);
    }

    pub fn balance_of(&self, token_name: String, owner: Address) -> U256 {
        let token = self.tokens.get(&token_name).unwrap_or_revert();
        token.balance_of(owner)
    }

    pub fn mint(&mut self, token_name: String, account: Address, amount: U256) {
        let mut token = self.tokens.get(&token_name).unwrap_or_revert();
        token.mint(account, amount);
    }

    pub fn tokens_count(&self) -> u32 {
        self.count.get_or_default()
    }
}

#[cfg(test)]
mod test {
    use super::TokenManagerDeployer;
    use odra::test_env::get_account;

    const PLS: &str = "PLS";
    const MCN: &str = "MCN";
    const PLASCOIN: &str = "Plascoin";
    const MY_COIN: &str = "MyCoin";
    const DECIMALS: u8 = 10;

    #[test]
    fn mappings() {
        let mut contract = TokenManagerDeployer::default();
        let (user1, user2, user3) = (get_account(0), get_account(1), get_account(2));
        let (pls_balance1, pls_balance2, pls_balance3) = (100.into(), 200.into(), 300.into());
        let (mcn_balance1, mcn_balance2, mcn_balance3) = (1000.into(), 2000.into(), 3000.into());

        contract.add_token(
            String::from(PLS),
            DECIMALS,
            String::from(PLS),
            String::from(PLASCOIN)
        );
        contract.add_token(
            String::from(MCN),
            DECIMALS,
            String::from(MCN),
            String::from(MY_COIN)
        );
        contract.mint(String::from(PLS), user1, pls_balance1);
        contract.mint(String::from(PLS), user2, pls_balance2);
        contract.mint(String::from(PLS), user3, pls_balance3);

        contract.mint(String::from(MCN), user1, mcn_balance1);
        contract.mint(String::from(MCN), user2, mcn_balance2);
        contract.mint(String::from(MCN), user3, mcn_balance3);

        assert_eq!(contract.balance_of(String::from(PLS), user1), pls_balance1);
        assert_eq!(contract.balance_of(String::from(PLS), user2), pls_balance2);
        assert_eq!(contract.balance_of(String::from(PLS), user3), pls_balance3);
        assert_eq!(contract.balance_of(String::from(MCN), user1), mcn_balance1);
        assert_eq!(contract.balance_of(String::from(MCN), user2), mcn_balance2);
        assert_eq!(contract.balance_of(String::from(MCN), user3), mcn_balance3);

        assert_eq!(contract.tokens_count(), 2);
    }
}
