use odra::{Variable, Mapping, types::{Address, Balance, event::OdraEvent}, contract_env};

use crate::traits::{Mintable, Burnable};

use self::{
    errors::Error,
    events::{Approval, Transfer},
};

#[odra::module]
pub struct Erc20 {
    decimals: Variable<u8>,
    symbol: Variable<String>,
    name: Variable<String>,
    total_supply: Variable<Balance>,
    balances: Mapping<Address, Balance>,
    allowances: Mapping<(Address, Address), Balance>,
}

#[odra::module]
impl Erc20 {
    #[odra(init)]
    pub fn init_with_supply(
        &self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: Balance,
    ) {
        let caller = contract_env::caller();

        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);
        self.balances.set(&caller, initial_supply);

        if initial_supply > Balance::zero() {
            Transfer { from: None, to: Some(caller), amount: initial_supply }.emit();
        }
    }

    #[odra(init)]
    pub fn init(&self, symbol: String, name: String, decimals: u8) {
        self.init_with_supply(symbol, name, decimals, Balance::zero());
    }

    pub fn transfer(&self, recipient: Address, amount: Balance) {
        let caller = contract_env::caller();
        self.raw_transfer(caller, recipient, amount);
    }

    pub fn transfer_from(&self, owner: Address, recipient: Address, amount: Balance) {
        let spender = contract_env::caller();
        self.spend_allowance(owner, spender, amount);

        self.raw_transfer(owner, recipient, amount);
    }

    pub fn approve(&self, spender: Address, amount: Balance) {
        let owner = contract_env::caller();

        self.allowances.set(&(owner, spender), amount);
        Approval { owner, spender, value: amount }.emit();
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn symbol(&self) -> String {
        self.symbol.get_or_default()
    }

    pub fn decimals(&self) -> u8 {
        self.decimals.get_or_default()
    }

    pub fn total_supply(&self) -> Balance {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, address: Address) -> Balance {
        self.balances.get_or_default(&address)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Balance {
        self.allowances.get_or_default(&(owner, spender))
    }
}

impl Mintable for Erc20 {
    fn mint(&self, address: Address, amount: Balance) {
        self.increase_total_supply(amount);
        self.increase_balance_of(&address, amount);

        Transfer { from: None, to: Some(address), amount }.emit();
    }
}

impl Burnable for Erc20 {
    fn burn(&self, address: Address, amount: Balance) {
        if self.balance_of(address) < amount {
            contract_env::revert(Error::InsufficientBalance);
        }
        self.decrease_total_supply(amount);
        self.decrease_balance_of(&address, amount);
    
        Transfer { from: Some(address), to: None, amount }.emit();
    }
}

impl Erc20 {
    fn raw_transfer(&self, owner: Address, recipient: Address, amount: Balance) {
        if amount > self.balances.get_or_default(&owner) {
            contract_env::revert(Error::InsufficientBalance)
        }

        self.balances.subtract(&owner, amount);
        self.balances.add(&recipient, amount);

        Transfer { from: Some(owner), to: Some(recipient), amount }.emit();
    }

    fn spend_allowance(&self, owner: Address, spender: Address, amount: Balance) {
        let key = (spender, owner);
        if self.allowances.get_or_default(&key) < amount {
            contract_env::revert(Error::InsufficientAllowance)
        }
        self.allowances.subtract(&key, amount);
        Approval { owner, spender, value: amount }.emit();
    }

    pub fn increase_total_supply(&self, amount: Balance) {
        self.total_supply.add(amount);
    }

    pub fn decrease_total_supply(&self, amount: Balance) {
        self.total_supply.subtract(amount);
    }

    pub fn increase_balance_of(&self, address: &Address, amount: Balance) {
        self.balances.add(address, amount);
    }

    pub fn decrease_balance_of(&self, address: &Address, amount: Balance) {
        self.balances.subtract(address, amount);
    }
}

pub mod events {
    use odra::Event;
    use odra::types::{Address, Balance};

    #[derive(Event, PartialEq, Debug)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub amount: Balance,
    }

    #[derive(Event, PartialEq, Debug)]
    pub struct Approval {
        pub owner: Address,
        pub spender: Address,
        pub value: Balance,
    }
}

pub mod errors {
    use odra::execution_error;

    execution_error! {
        pub enum Error {
            InsufficientBalance => 20,
            InsufficientAllowance => 21,
        }
    }
}

#[cfg(all(test, feature = "mock-vm"))]
mod tests {

    use odra::{assert_events, test_env, types::Balance};

    use crate::erc20::events::Transfer;

    use super::{errors::Error, Erc20, Erc20Ref};

    const NAME: &str = "Plascoin";
    const SYMBOL: &str = "PLS";
    const DECIMALS: u8 = 10;
    const INITIAL_SUPPLY: u32 = 10_000;

    fn setup() -> Erc20Ref {
        let erc20 = Erc20::deploy_init_with_supply(
            SYMBOL.to_string(),
            NAME.to_string(),
            DECIMALS,
            INITIAL_SUPPLY.into(),
        );
        erc20
    }

    #[test]
    fn initialization() {
        let erc20 = setup();

        assert_eq!(erc20.symbol(), SYMBOL.to_string());
        assert_eq!(erc20.name(), NAME.to_string());
        assert_eq!(erc20.decimals(), DECIMALS);
        assert_eq!(erc20.total_supply(), INITIAL_SUPPLY.into());

        assert_events!(
            erc20,
            Transfer {
                from: None,
                to: Some(test_env::get_account(0)),
                amount: INITIAL_SUPPLY.into()
            }
        );
    }

    #[test]
    fn transfer_works() {
        let erc20 = setup();

        let sender = test_env::get_account(0);
        let recipient = test_env::get_account(1);

        let amount = 1_000.into();

        erc20.transfer(recipient, amount);

        assert_eq!(
            erc20.balance_of(sender),
            Balance::from(INITIAL_SUPPLY) - amount
        );

        assert_eq!(erc20.balance_of(recipient), amount);

        assert_events!(
            erc20,
            Transfer {
                from: Some(sender),
                to: Some(recipient),
                amount
            }
        );
    }

    #[test]
    fn transfer_error() {
        let erc20 = setup();

        let recipient = test_env::get_account(1);

        let amount = Balance::from(INITIAL_SUPPLY) + Balance::one();

        test_env::assert_exception(Error::InsufficientBalance, || {
            erc20.transfer(recipient, amount)
        });
    }
}