use odra_env::ContractEnv;
use odra_primitives::{Mapping, Variable};
use odra_types::{Address, U256};

use self::{
    errors::Error,
    events::{Approval, Transfer},
};

pub mod ext;

#[odra_proc_macros::module]
pub struct Erc20 {
    decimals: Variable<u8>,
    symbol: Variable<String>,
    name: Variable<String>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<(Address, Address), U256>,
}

#[odra_proc_macros::module]
impl Erc20 {
    #[odra(init)]
    pub fn init_with_supply(
        &self,
        symbol: String,
        name: String,
        decimals: u8,
        initial_supply: U256,
    ) {
        let caller = ContractEnv::caller();

        self.symbol.set(symbol);
        self.name.set(name);
        self.decimals.set(decimals);
        self.total_supply.set(initial_supply);
        self.balances.set(&caller, initial_supply);

        Transfer::emit(None, Some(caller), initial_supply);
    }

    #[odra(init)]
    pub fn init(&self, symbol: String, name: String, decimals: u8) {
        self.init_with_supply(symbol, name, decimals, U256::zero());
    }

    pub fn transfer(&self, recipient: Address, amount: U256) {
        let caller = ContractEnv::caller();
        self.raw_transfer(caller, recipient, amount);
    }

    pub fn transfer_from(&self, owner: Address, recipient: Address, amount: U256) {
        let spender = ContractEnv::caller();
        self.spend_allowance(owner, spender, amount);

        self.raw_transfer(owner, recipient, amount);
    }

    pub fn approve(&self, spender: Address, amount: U256) {
        let owner = ContractEnv::caller();

        self.allowances.set(&(owner, spender), amount);
        Approval::emit(owner, spender, amount);
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

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, address: Address) -> U256 {
        self.balances.get_or_default(&address)
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.get_or_default(&(owner, spender))
    }
}

impl Erc20 {
    fn raw_transfer(&self, owner: Address, recipient: Address, amount: U256) {
        if amount > self.balances.get_or_default(&owner) {
            ContractEnv::revert(Error::InsufficientBalance)
        }

        self.balances.subtract(&owner, amount);
        self.balances.add(&recipient, amount);

        Transfer::emit(Some(owner), Some(recipient), amount);
    }

    fn spend_allowance(&self, owner: Address, spender: Address, amount: U256) {
        let key = (spender, owner);
        if self.allowances.get_or_default(&key) < amount {
            ContractEnv::revert(Error::InsufficientAllowance)
        }
        self.allowances.subtract(&key, amount);
        Approval::emit(owner, spender, amount);
    }

    pub fn increase_total_supply(&self, amount: U256) {
        self.total_supply.add(amount);
    }

    pub fn decrease_total_supply(&self, amount: U256) {
        self.total_supply.subtract(amount);
    }

    pub fn increase_balance_of(&self, address: &Address, amount: U256) {
        self.balances.add(address, amount);
    }

    pub fn decrease_balance_of(&self, address: &Address, amount: U256) {
        self.balances.subtract(address, amount);
    }
}

pub mod events {
    use odra_proc_macros::Event;
    use odra_types::{event::Event, Address, U256};

    #[derive(Event, PartialEq, Debug)]
    pub struct Transfer {
        pub from: Option<Address>,
        pub to: Option<Address>,
        pub amount: U256,
    }

    impl Transfer {
        pub fn emit(from: Option<Address>, to: Option<Address>, amount: U256) {
            Self { from, to, amount }.emit()
        }
    }

    #[derive(Event, PartialEq, Debug)]
    pub struct Approval {
        pub owner: Address,
        pub spender: Address,
        pub value: U256,
    }

    impl Approval {
        pub fn emit(owner: Address, spender: Address, value: U256) {
            Self {
                owner,
                spender,
                value,
            }
            .emit()
        }
    }
}

pub mod errors {
    use odra_proc_macros::execution_error;

    execution_error! {
        pub enum Error {
            InsufficientBalance => 20,
            InsufficientAllowance => 21,
        }
    }
}

#[cfg(all(test, feature = "mock-vm"))]
mod tests {

    use odra_env::{assert_events, TestEnv};
    use odra_types::U256;

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
                to: Some(TestEnv::get_account(0)),
                amount: INITIAL_SUPPLY.into()
            }
        );
    }

    #[test]
    fn transfer_works() {
        let erc20 = setup();

        let sender = TestEnv::get_account(0);
        let recipient = TestEnv::get_account(1);

        let amount = 1_000.into();

        erc20.transfer(recipient, amount);

        assert_eq!(
            erc20.balance_of(sender),
            U256::from(INITIAL_SUPPLY) - amount
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

        let recipient = TestEnv::get_account(1);

        let amount = U256::from(INITIAL_SUPPLY) + U256::one();

        TestEnv::assert_exception(Error::InsufficientBalance, || {
            erc20.transfer(recipient, amount)
        });
    }
}
