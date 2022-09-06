use odra_env::ContractEnv;
use odra_types::{Address, U256, event::Event};

use crate::erc20::{Erc20, events::Transfer, errors::Error};

pub fn mint(erc20: &Erc20, address: Address, amount: U256) {
    erc20.increase_total_supply(amount);
    erc20.increase_balance_of(&address, amount);

    Transfer { from: None, to: Some(address), amount }.emit();
}

pub fn burn(erc20: &Erc20, address: Address, amount: U256) {
    if erc20.balance_of(address) < amount {
        ContractEnv::revert(Error::InsufficientBalance);
    }
    erc20.increase_total_supply(amount);
    erc20.increase_balance_of(&address, amount);
}
