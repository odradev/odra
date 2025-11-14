use odra::{casper_types::U256, prelude::*};
use odra_modules::{access::Ownable, cep18_token::Cep18};

#[odra::module(factory=on)]
pub struct FToken {
    /// The CEP-18 token submodule
    token: SubModule<Cep18>,
    /// The Ownable submodule
    ownable: SubModule<Ownable>
}

#[odra::module(factory=on)]
impl FToken {
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        self.token.init(symbol, name, decimals, initial_supply);
        let owner = self.env().caller();
        self.ownable.init(owner);
    }

    delegate! {
        to self.token {
            fn transfer(&mut self, to: &Address, amount: &U256);
            fn balance_of(&self, owner: &Address) -> U256;
            fn total_supply(&self) -> U256;
            fn name(&self) -> String;
            fn symbol(&self) -> String;
        }

        to self.ownable {
            fn get_owner(&self) -> Address;
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use odra::{
        casper_types::U256,
        host::{Deployer, HostRef, NoArgs}
    };

    use crate::factory::token::{
        FToken as Token, FTokenFactory as TokenFactory, FTokenHostRef,
        FTokenInitArgs as TokenInitArgs
    };

    #[test]
    fn test_standalone_module() {
        let env = odra_test::env();
        let owner = env.get_account(0);
        let token = Token::deploy(
            &env,
            TokenInitArgs {
                name: "MyToken".to_string(),
                symbol: "MTK".to_string(),
                decimals: 18,
                initial_supply: U256::from(1000u64)
            }
        );
        assert_eq!(token.get_owner(), owner);
        assert_eq!(token.name(), "MyToken".to_string());
        assert_eq!(token.symbol(), "MTK".to_string());
        assert_eq!(token.total_supply(), U256::from(1000u64));
    }

    #[test]
    #[ignore = "This test does not work on odra vm"]
    fn test_factory_module() {
        let env = odra_test::env();
        let owner = env.get_account(0);
        let mut factory = TokenFactory::deploy(&env, NoArgs);

        let (addr, _) = factory.new_contract(
            "TokenContract".to_string(),
            "Token".to_string(),
            "TTK".to_string(),
            18,
            U256::from(500u64)
        );
        let token = FTokenHostRef::new(addr, env);
        assert_eq!(token.get_owner(), owner);
        assert_eq!(token.name(), "Token".to_string());
        assert_eq!(token.symbol(), "TTK".to_string());
        assert_eq!(token.total_supply(), U256::from(500u64));
    }
}
