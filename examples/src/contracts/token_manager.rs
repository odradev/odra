use crate::contracts::owned_token::OwnedToken;
use odra::prelude::*;
use odra::{Address, Mapping, ModuleWrapper, Variable, U256};

#[odra::module]
pub struct TokenManager {
    tokens: Mapping<String, OwnedToken>,
    count: Variable<u32>
}

#[odra::module]
impl TokenManager {
    pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
        self.tokens
            .module(&name)
            .init(name, symbol, decimals, U256::from(0));

        let new_count = self.count.get_or_default() + 1;
        self.count.set(new_count);
    }

    pub fn balance_of(&self, token_name: String, owner: &Address) -> U256 {
        self.get_token(token_name).balance_of(owner)
    }

    pub fn mint(&mut self, token_name: String, account: &Address, amount: &U256) {
        self.get_token(token_name).mint(account, amount);
    }

    pub fn tokens_count(&self) -> u32 {
        self.count.get_or_default()
    }

    pub fn get_owner(&self, token_name: String) -> Address {
        self.get_token(token_name).get_owner()
    }

    pub fn set_owner(&mut self, token_name: String, new_owner: &Address) {
        self.get_token(token_name).transfer_ownership(new_owner);
    }

    fn get_token(&self, token_name: String) -> ModuleWrapper<OwnedToken> {
        self.tokens.module(&token_name)
    }
}

#[cfg(test)]
mod test {
    use super::{TokenManagerDeployer, TokenManagerHostRef};
    use odra::prelude::*;

    const PLS: &str = "PLS";
    const MCN: &str = "MCN";
    const PLASCOIN: &str = "Plascoin";
    const MY_COIN: &str = "MyCoin";
    const DECIMALS: u8 = 10;

    fn setup() -> TokenManagerHostRef {
        let test_env = odra_test::test_env();
        let mut contract = TokenManagerDeployer::init(&test_env);

        contract.add_token(String::from(PLASCOIN), DECIMALS, String::from(PLS));
        contract.add_token(String::from(MY_COIN), DECIMALS, String::from(MCN));
        contract
    }

    #[test]
    fn minting_works() {
        let mut contract = setup();
        let test_env = contract.env().clone();
        let (user1, user2, user3) = (
            test_env.get_account(0),
            test_env.get_account(1),
            test_env.get_account(2)
        );
        let (pls_balance1, pls_balance2, pls_balance3) = (100.into(), 200.into(), 300.into());
        let (mcn_balance1, mcn_balance2, mcn_balance3) = (1000.into(), 2000.into(), 3000.into());

        let plascoin = String::from(PLASCOIN);
        let my_coin = String::from(MY_COIN);

        contract.mint(plascoin.clone(), user1, pls_balance1);
        contract.mint(plascoin.clone(), user2, pls_balance2);
        contract.mint(plascoin.clone(), user3, pls_balance3);

        contract.mint(my_coin.clone(), user1, mcn_balance1);
        contract.mint(my_coin.clone(), user2, mcn_balance2);
        contract.mint(my_coin.clone(), user3, mcn_balance3);

        assert_eq!(contract.balance_of(plascoin.clone(), user1), pls_balance1);
        assert_eq!(contract.balance_of(plascoin.clone(), user2), pls_balance2);
        assert_eq!(contract.balance_of(plascoin, user3), pls_balance3);
        assert_eq!(contract.balance_of(my_coin.clone(), user1), mcn_balance1);
        assert_eq!(contract.balance_of(my_coin.clone(), user2), mcn_balance2);
        assert_eq!(contract.balance_of(my_coin, user3), mcn_balance3);
    }

    #[test]
    fn tokens_count_works() {
        let contract = setup();
        assert_eq!(contract.tokens_count(), 2);
    }

    #[test]
    fn get_owner_works() {
        let mut contract = setup();
        let test_env = contract.env().clone();
        let (owner, user2, user3) = (
            test_env.get_account(0),
            test_env.get_account(1),
            test_env.get_account(2)
        );

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), owner);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), owner);

        contract.set_owner(String::from(PLASCOIN), user2);
        contract.set_owner(String::from(MY_COIN), user3);

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), user2);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), user3);
    }

    #[test]
    fn many_tokens_works() {
        let test_env = odra_test::test_env();
        let mut contract = TokenManagerDeployer::init(&test_env);
        let (user, balance) = (test_env.get_account(0), 111.into());
        for i in 0..20 {
            contract.add_token(i.to_string(), DECIMALS, i.to_string());
            contract.mint(i.to_string(), user, balance);
            assert_eq!(contract.balance_of(i.to_string(), user), balance);
        }
    }
}
