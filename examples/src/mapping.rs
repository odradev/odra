use odra::{
    map,
    types::{Address, U256},
    Instance, Mapping, Variable
};

use crate::owned_token::OwnedToken;

#[odra::module]
pub struct NestedMapping {
    value: Mapping<String, Mapping<u32, Mapping<String, String>>>,
    value2: Mapping<String, Mapping<u32, Mapping<String, OwnedToken>>>
}

#[odra::module]
impl NestedMapping {
    pub fn set(&mut self, key1: String, key2: u32, key3: String, value: String) {
        map!(self.value[key1][key2][key3] = value.clone());
        map!(self.value2[key1][key2][key3] = OwnedToken::instance(&value));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_token(
        &mut self,
        key1: String,
        key2: u32,
        key3: String,
        token_name: String,
        decimals: u8,
        symbol: String,
        initial_supply: U256
    ) {
        let mut token = OwnedToken::instance(&token_name);
        token.init(token_name, symbol, decimals, initial_supply);
        map!(self.value2[key1][key2][key3] = token);
    }

    pub fn get(&self, key1: String, key2: u32, key3: String) -> String {
        map!(self.value[key1][key2][key3])
    }

    pub fn get_manually(&self, key1: String, key2: u32, key3: String) -> String {
        let lvl1 = self.value.get_instance(&key1);
        let lvl2 = lvl1.get_instance(&key2);
        odra::UnwrapOrRevert::unwrap_or_revert(lvl2.get(&key3))
    }

    pub fn total_supply(&self, key1: String, key2: u32, key3: String) -> U256 {
        map!(self.value2[key1][key2][key3]).total_supply()
    }
}

#[odra::module]
pub struct TokenManager {
    tokens: Mapping<String, OwnedToken>,
    count: Variable<u32>
}

#[odra::module]
impl TokenManager {
    pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
        let count = self.count.get_or_default();

        let mut module = self.tokens.get_instance(&name);
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
        self.tokens.get_instance(&token_name)
    }
}

#[cfg(test)]
mod test {
    use crate::mapping::NestedMappingDeployer;

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
    #[ignore]
    fn many_tokens_works() {
        let mut contract = TokenManagerDeployer::default();
        let (user, balance) = (get_account(0), 111.into());
        for i in 0..500 {
            contract.add_token(i.to_string(), DECIMALS, i.to_string());
            contract.mint(i.to_string(), user, balance);
            assert_eq!(contract.balance_of(i.to_string(), user), balance);
        }
    }

    #[test]
    fn nested_mapping_works() {
        // given a nested mapping contract
        let mut contract = NestedMappingDeployer::default();
        let (key1, key2, key3) = (String::from("a"), 1, String::from("b"));
        // when set a value
        let value = String::from("value");
        contract.set(key1.clone(), key2, key3.clone(), value.clone());
        // then the value can be retrieved using both get and get_manually
        assert_eq!(contract.get(key1.clone(), key2, key3.clone()), value);
        assert_eq!(
            contract.get_manually(key1.clone(), key2, key3.clone()),
            value
        );

        // when create a token
        let token_name = String::from("token");
        let decimals = 10;
        let symbol = String::from("SYM");
        let initial_supply = 100.into();
        contract.set_token(
            key1.clone(),
            key2,
            key3.clone(),
            token_name,
            decimals,
            symbol,
            initial_supply
        );
        // then the total supply is set
        assert_eq!(contract.total_supply(key1, key2, key3), initial_supply);
    }
}
