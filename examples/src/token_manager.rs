use odra::{
    types::{Address, U256},
    Mapping, Variable
};

use crate::owned_token::OwnedToken;

#[odra::module]
pub struct TokenManager {
    tokens: Mapping<String, OwnedToken>,
    count: Variable<u32>
}

#[odra::module]
impl TokenManager {
    pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
        self.tokens
            .get_instance(&name)
            .init(name.clone(), symbol, decimals, U256::from(0));

        self.count.set(self.count.get_or_default() + 1);
    }

    pub fn balance_of(&self, token_name: String, owner: Address) -> U256 {
        self.get_token(token_name).balance_of(owner)
    }

    pub fn mint(&mut self, token_name: String, account: Address, amount: U256) {
        self.get_token(token_name).mint(account, amount);
    }

    pub fn tokens_count(&self) -> u32 {
        self.count.get_or_default()
    }

    pub fn get_owner(&self, token_name: String) -> Address {
        self.get_token(token_name).get_owner()
    }

    pub fn set_owner(&mut self, token_name: String, new_owner: Address) {
        self.get_token(token_name).change_ownership(new_owner);
    }

    fn get_token(&self, token_name: String) -> OwnedToken {
        self.tokens.get_instance(&token_name)
    }
}

#[cfg(test)]
mod test {
    use super::{TokenManagerDeployer, TokenManagerRef};
    use odra::test_env::get_account;

    const PLS: &str = "PLS";
    const MCN: &str = "MCN";
    const PLASCOIN: &str = "Plascoin";
    const MY_COIN: &str = "MyCoin";
    const DECIMALS: u8 = 10;

    fn setup() -> TokenManagerRef {
        let mut contract = TokenManagerDeployer::default();

        contract.add_token(String::from(PLASCOIN), DECIMALS, String::from(PLS));
        contract.add_token(String::from(MY_COIN), DECIMALS, String::from(MCN));
        contract
    }

    #[test]
    fn minting_works() {
        let mut contract = setup();
        let (user1, user2, user3) = (get_account(0), get_account(1), get_account(2));
        let (pls_balance1, pls_balance2, pls_balance3) = (100.into(), 200.into(), 300.into());
        let (mcn_balance1, mcn_balance2, mcn_balance3) = (1000.into(), 2000.into(), 3000.into());

        contract.mint(String::from(PLASCOIN), user1, pls_balance1);
        contract.mint(String::from(PLASCOIN), user2, pls_balance2);
        contract.mint(String::from(PLASCOIN), user3, pls_balance3);

        contract.mint(String::from(MY_COIN), user1, mcn_balance1);
        contract.mint(String::from(MY_COIN), user2, mcn_balance2);
        contract.mint(String::from(MY_COIN), user3, mcn_balance3);

        assert_eq!(
            contract.balance_of(String::from(PLASCOIN), user1),
            pls_balance1
        );
        assert_eq!(
            contract.balance_of(String::from(PLASCOIN), user2),
            pls_balance2
        );
        assert_eq!(
            contract.balance_of(String::from(PLASCOIN), user3),
            pls_balance3
        );
        assert_eq!(
            contract.balance_of(String::from(MY_COIN), user1),
            mcn_balance1
        );
        assert_eq!(
            contract.balance_of(String::from(MY_COIN), user2),
            mcn_balance2
        );
        assert_eq!(
            contract.balance_of(String::from(MY_COIN), user3),
            mcn_balance3
        );
    }

    #[test]
    fn tokens_count_works() {
        let contract = setup();
        assert_eq!(contract.tokens_count(), 2);
    }

    #[test]
    fn get_owner_works() {
        let mut contract = setup();
        let (owner, user2, user3) = (get_account(0), get_account(1), get_account(2));

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), owner);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), owner);

        contract.set_owner(String::from(PLASCOIN), user2);
        contract.set_owner(String::from(MY_COIN), user3);

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), user2);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), user3);
    }

    #[test]
    fn many_tokens_works() {
        let mut contract = TokenManagerDeployer::default();
        let (user, balance) = (get_account(0), 111.into());
        for i in 0..20 {
            contract.add_token(i.to_string(), DECIMALS, i.to_string());
            contract.mint(i.to_string(), user, balance);
            assert_eq!(contract.balance_of(i.to_string(), user), balance);
        }
    }
}