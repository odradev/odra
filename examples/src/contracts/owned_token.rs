//! An example of a OwnedToken contract.
use odra::casper_types::U256;
use odra::prelude::*;
use odra_modules::access::Ownable;
use odra_modules::erc20::Erc20;

/// OwnedToken contract.
#[odra::module(name = "MyTokenContact", version = "1.0.0")]
pub struct OwnedToken {
    ownable: SubModule<Ownable>,
    erc20: SubModule<Erc20>
}

#[odra::module]
impl OwnedToken {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        self.ownable.init();
        self.erc20
            .init(symbol, name, decimals, Some(initial_supply));
    }

    delegate! {
        to self.erc20 {
            fn transfer(&mut self, recipient: &Address, amount: &U256);
            fn transfer_from(&mut self, owner: &Address, recipient: &Address, amount: &U256);
            fn approve(&mut self, spender: &Address, amount: &U256);
            fn name(&self) -> String;
            fn symbol(&self) -> String;
            fn decimals(&self) -> u8;
            fn total_supply(&self) -> U256;
            fn balance_of(&self, owner: &Address) -> U256;
            fn allowance(&self, owner: &Address, spender: &Address) -> U256;
        }
    }

    delegate! {
        to self.ownable {
            fn get_owner(&self) -> Address;
            fn transfer_ownership(&mut self, new_owner: &Address);
        }
    }

    /// Mints new tokens and assigns them to the given address.
    pub fn mint(&mut self, address: &Address, amount: &U256) {
        self.ownable.assert_owner(&self.env().caller());
        self.erc20.mint(address, amount);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use odra::{
        host::{Deployer, HostRef},
        VmError
    };
    use odra_modules::access::errors::Error::CallerNotTheOwner;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;

    pub fn setup() -> OwnedTokenHostRef {
        let init_args = OwnedTokenInitArgs {
            name: String::from(NAME),
            symbol: String::from(SYMBOL),
            decimals: DECIMALS,
            initial_supply: INITIAL_SUPPLY.into()
        };
        OwnedToken::deploy(&odra_test::env(), init_args)
    }

    #[test]
    fn init_works() {
        let token = setup();
        let test_env = token.env().clone();
        let owner = test_env.get_account(0);
        assert_eq!(token.symbol(), SYMBOL);
        assert_eq!(token.decimals(), DECIMALS);
        assert_eq!(token.total_supply(), INITIAL_SUPPLY.into());
        assert_eq!(token.balance_of(&owner), INITIAL_SUPPLY.into());
        test_env.emitted_event(
            &token,
            &odra_modules::access::events::OwnershipTransferred {
                previous_owner: None,
                new_owner: Some(owner)
            }
        );
        test_env.emitted_event(
            &token,
            &odra_modules::erc20::events::Transfer {
                from: None,
                to: Some(owner),
                amount: INITIAL_SUPPLY.into()
            }
        );
    }

    #[test]
    fn should_not_init_twice() {
        let mut token = setup();
        let result = token.try_init(
            String::from(NAME),
            String::from(SYMBOL),
            11u8,
            INITIAL_SUPPLY.into()
        );
        assert_eq!(
            result.unwrap_err(),
            OdraError::VmError(VmError::InvalidContext)
        );
    }

    #[test]
    fn mint_works() {
        let mut token = setup();
        let recipient = token.env().get_account(1);
        let amount = 10.into();
        token.mint(&recipient, &amount);
        assert_eq!(token.total_supply(), U256::from(INITIAL_SUPPLY) + amount);
        assert_eq!(&token.balance_of(&recipient), &amount);
    }

    #[test]
    fn mint_error() {
        let mut token = setup();
        let test_env = token.env().clone();
        let recipient = test_env.get_account(1);
        let amount = 10.into();
        test_env.set_caller(recipient);
        assert_eq!(
            token.try_mint(&recipient, &amount).unwrap_err(),
            CallerNotTheOwner.into()
        );
    }

    #[test]
    fn change_ownership_works() {
        let mut token = setup();
        let new_owner = token.env().get_account(1);
        token.transfer_ownership(&new_owner);
        assert_eq!(token.get_owner(), new_owner);
    }

    #[test]
    fn change_ownership_error() {
        let mut token = setup();
        let test_env = token.env().clone();
        let new_owner = test_env.get_account(1);
        test_env.set_caller(new_owner);
        assert_eq!(
            token.try_transfer_ownership(&new_owner).unwrap_err(),
            CallerNotTheOwner.into()
        );
    }
}
