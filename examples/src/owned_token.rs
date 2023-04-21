use odra::{
    contract_env,
    types::{Address, U256}, Instance
};

use crate::{erc20::{Erc20, Erc20Composer}, ownable::{Ownable, OwnableComposer}};

#[odra::module(skip_instance)]
pub struct OwnedToken {
    ownable: Ownable,
    erc20: Erc20
}

impl Instance for OwnedToken {
    fn instance(namespace: &str) -> Self {
        Self {
            ownable: OwnableComposer::new(namespace).compose(),
            erc20: Erc20Composer::new(namespace).compose(),
        }
    }
}

#[odra::module]
impl OwnedToken {
    #[odra(init)]
    pub fn init(&mut self, name: String, symbol: String, decimals: u8, initial_supply: U256) {
        let deployer = contract_env::caller();
        self.ownable.init(deployer);
        self.erc20.init(name, symbol, decimals, initial_supply);
    }

    delegate! {
        to self.erc20 {
            pub fn transfer(&mut self, recipient: Address, amount: U256);
            pub fn transfer_from(&mut self, owner: Address, recipient: Address, amount: U256);
            pub fn approve(&mut self, spender: Address, amount: U256);
            pub fn name(&self) -> String;
            pub fn symbol(&self) -> String;
            pub fn decimals(&self) -> u8;
            pub fn total_supply(&self) -> U256;
            pub fn balance_of(&self, owner: Address) -> U256;
            pub fn allowance(&self, owner: Address, spender: Address) -> U256;
        }

        to self.ownable {
            pub fn get_owner(&self) -> Address;
            pub fn change_ownership(&mut self, new_owner: Address);
        }
    }

    pub fn mint(&mut self, address: Address, amount: U256) {
        self.ownable.ensure_ownership(contract_env::caller());
        self.erc20.mint(address, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{erc20, ownable};
    use odra::{assert_events, test_env, types::U256};

    const NAME: &str = "Plascoin";
    const SYMBOL: &str = "PLS";
    const DECIMALS: u8 = 10;
    const INITIAL_SUPPLY: u32 = 10_000;

    fn setup() -> OwnedTokenRef {
        OwnedTokenDeployer::init(
            String::from(NAME),
            String::from(SYMBOL),
            DECIMALS,
            INITIAL_SUPPLY.into()
        )
    }

    #[test]
    fn init_works() {
        let token = setup();
        let owner = test_env::get_account(0);
        assert_eq!(&token.symbol(), SYMBOL);
        assert_eq!(token.decimals(), DECIMALS);
        assert_eq!(token.total_supply(), INITIAL_SUPPLY.into());
        assert_eq!(token.balance_of(owner), INITIAL_SUPPLY.into());
        assert_events!(
            token,
            ownable::OwnershipChanged {
                prev_owner: None,
                new_owner: owner
            },
            erc20::Transfer {
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
        token.mint(recipient, amount);
        assert_eq!(token.total_supply(), U256::from(INITIAL_SUPPLY) + amount);
        assert_eq!(token.balance_of(recipient), amount);
    }

    #[test]
    fn mint_error() {
        let token = setup();
        let recipient = test_env::get_account(1);
        let amount = 10.into();
        test_env::set_caller(recipient);
        test_env::assert_exception(ownable::Error::NotOwner, || {
            // TODO: If we don't create a new ref, an error occurs:
            // cannot borrow `token` as mutable, as it is a captured variable in a `Fn` closure cannot borrow as mutable
            let mut token = OwnedTokenRef::at(token.address());
            token.mint(recipient, amount);
        });
    }

    #[test]
    fn change_ownership_works() {
        let mut token = setup();
        let new_owner = test_env::get_account(1);
        token.change_ownership(new_owner);
        assert_eq!(token.get_owner(), new_owner);
    }

    #[test]
    fn change_ownership_error() {
        let token = setup();
        let new_owner = test_env::get_account(1);
        test_env::set_caller(new_owner);
        test_env::assert_exception(ownable::Error::NotOwner, || {
            // TODO: If we don't create a new ref, an error occurs:
            // cannot borrow `token` as mutable, as it is a captured variable in a `Fn` closure cannot borrow as mutable
            let mut token = OwnedTokenRef::at(token.address());
            token.change_ownership(new_owner)
        });
    }
}
