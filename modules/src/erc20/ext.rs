use odra_env::ContractEnv;
use odra_types::{event::Event, Address, U256};

use crate::erc20::{errors::Error, events::Transfer, Erc20};

pub fn mint(erc20: &Erc20, address: Address, amount: U256) {
    erc20.increase_total_supply(amount);
    erc20.increase_balance_of(&address, amount);

    Transfer {
        from: None,
        to: Some(address),
        amount,
    }
    .emit();
}

pub fn burn(erc20: &Erc20, address: Address, amount: U256) {
    if erc20.balance_of(address) < amount {
        ContractEnv::revert(Error::InsufficientBalance);
    }
    erc20.increase_total_supply(amount);
    erc20.increase_balance_of(&address, amount);
}
