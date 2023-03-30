use odra::{
    types::{Address, U256},
    Instance, Mapping, UnwrapOrRevert, Variable
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
        let count = self.count.get_or_default();
        let mut module = OwnedToken::instance(&name);
        module.init(name.clone(), symbol, decimals, U256::from(0));

        self.tokens.set(&name, module);
        self.count.set(count + 1);
    }

    pub fn balance_of(&self, token_name: String, owner: Address) -> U256 {
        let token = self.get_token(token_name);
        token.balance_of(owner)
    }

    pub fn mint(&mut self, token_name: String, account: Address, amount: U256) {
        let mut token = self.get_token(token_name);
        token.mint(account, amount);
    }

    pub fn tokens_count(&self) -> u32 {
        self.count.get_or_default()
    }

    pub fn get_owner(&self, token_name: String) -> Address {
        let token = self.get_token(token_name);
        token.get_owner()
    }

    pub fn set_owner(&mut self, token_name: String, new_owner: Address) {
        let mut token = self.get_token(token_name);
        token.change_ownership(new_owner);
    }

    fn get_token(&self, token_name: String) -> OwnedToken {
        self.tokens.get(&token_name).unwrap_or_revert()
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
}
