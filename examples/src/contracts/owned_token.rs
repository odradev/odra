use odra::{Address, Module, ModuleWrapper, U256};
use odra::prelude::*;
use odra_modules::access::Ownable;
use odra_modules::erc20::Erc20;

#[odra::module]
pub struct OwnedToken {
    ownable: ModuleWrapper<Ownable>,
    erc20: ModuleWrapper<Erc20>
}

#[odra::module]
impl OwnedToken {
    #[odra(init)]
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        self.ownable.init();
        self.erc20
            .init(symbol, name, decimals, Some(initial_supply));
    }

    delegate! {
        to self.erc20 {
            pub fn transfer(&mut self, recipient: &Address, amount: &U256);
            pub fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
            pub fn approve(&mut self, spender: &Address, amount: &U256);
            pub fn name(&self) -> String;
            pub fn symbol(&self) -> String;
            pub fn decimals(&self) -> u8;
            pub fn total_supply(&self) -> U256;
            pub fn balance_of(&self, owner: &Address) -> U256;
            pub fn allowance(&self, owner: &Address, spender: &Address) -> U256;
        }

        to self.ownable {
            pub fn get_owner(&self) -> Address;
            pub fn transfer_ownership(&mut self, new_owner: &Address);
        }
    }

    pub fn mint(&mut self, address: &Address, amount: &U256) {
        self.ownable.assert_owner(&self.env().caller());
        self.erc20.mint(address, amount);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use odra::{assert_events, test_env, types::casper_types::U256};
    use odra_modules::access::errors::Error;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;

    pub fn setup() -> OwnedTokenRef {
        OwnedTokenDeployer::init(
            String::from(NAME),
            String::from(SYMBOL),
            DECIMALS,
            &INITIAL_SUPPLY.into()
        )
    }

    #[test]
    fn init_works() {
        let token = setup();
        let owner = test_env::get_account(0);
        assert_eq!(&token.symbol(), SYMBOL);
        assert_eq!(token.decimals(), DECIMALS);
        assert_eq!(token.total_supply(), INITIAL_SUPPLY.into());
        assert_eq!(token.balance_of(&owner), INITIAL_SUPPLY.into());
        assert_events!(
            token,
            odra_modules::access::events::OwnershipTransferred {
                previous_owner: None,
                new_owner: Some(owner)
            },
            odra_modules::erc20::events::Transfer {
                from: None,
                to: Some(owner),
                amount: INITIAL_SUPPLY.into()
            }
        );
    }

    #[test]
    fn mint_works() {
        let mut token = setup();
        let recipient = test_env::get_account(1);
        let amount = 10.into();
        token.mint(&recipient, &amount);
        assert_eq!(token.total_supply(), U256::from(INITIAL_SUPPLY) + amount);
        assert_eq!(token.balance_of(&recipient), amount);
    }

    #[test]
    fn mint_error() {
        let mut token = setup();
        let recipient = test_env::get_account(1);
        let amount = 10.into();
        test_env::set_caller(recipient);
        test_env::assert_exception(Error::CallerNotTheOwner, || token.mint(&recipient, &amount));
    }

    #[test]
    fn change_ownership_works() {
        let mut token = setup();
        let new_owner = test_env::get_account(1);
        token.transfer_ownership(&new_owner);
        assert_eq!(token.get_owner(), new_owner);
    }

    #[test]
    fn change_ownership_error() {
        let mut token = setup();
        let new_owner = test_env::get_account(1);
        test_env::set_caller(new_owner);
        test_env::assert_exception(Error::CallerNotTheOwner, || {
            token.transfer_ownership(&new_owner)
        });
    }
}
