#![allow(missing_docs)]

use odra::casper_types::bytesrepr::ToBytes;
use odra::casper_types::U256;
use odra::named_keys::{
    base64_encoded_key_value_storage, compound_key_value_storage, single_value_storage
};
use odra::prelude::*;

use crate::cep18::errors::Error::{InvalidState, Overflow};

const ALLOWANCES_KEY: &str = "allowances";
const BALANCES_KEY: &str = "balances";
const NAME_KEY: &str = "name";
const DECIMALS_KEY: &str = "decimals";
const SYMBOL_KEY: &str = "symbol";
const TOTAL_SUPPLY_KEY: &str = "total_supply";

single_value_storage!(Cep18NameStorage, String, NAME_KEY, InvalidState);
single_value_storage!(Cep18DecimalsStorage, u8, DECIMALS_KEY, InvalidState);
single_value_storage!(Cep18SymbolStorage, String, SYMBOL_KEY, InvalidState);
single_value_storage!(
    Cep18TotalSupplyStorage,
    U256,
    TOTAL_SUPPLY_KEY,
    InvalidState
);
impl Cep18TotalSupplyStorage {
    /// Adds the given amount to the total supply of the token.
    pub fn add(&self, amount: U256) {
        let total_supply = self.get();
        let new_total_supply = total_supply
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), ExecutionError::AdditionOverflow);
        self.set(new_total_supply);
    }

    /// Subtracts the given amount from the total supply of the token.
    pub fn subtract(&self, amount: U256) {
        let total_supply = self.get();
        let new_total_supply = total_supply
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), Overflow);
        self.set(new_total_supply);
    }
}
base64_encoded_key_value_storage!(Cep18BalancesStorage, BALANCES_KEY, Address, U256);
impl Cep18BalancesStorage {
    /// Adds the given amount to the balance of the given account.
    pub fn add(&self, account: &Address, amount: U256) {
        let balance = self.get(account).unwrap_or_default();
        let new_balance = balance.checked_add(amount).unwrap_or_revert(self);
        self.set(account, new_balance);
    }

    /// Subtracts the given amount from the balance of the given account.
    pub fn subtract(&self, account: &Address, amount: U256) {
        let balance = self.get(account).unwrap_or_default();
        let new_balance = balance
            .checked_sub(amount)
            .unwrap_or_revert_with(self, Overflow);
        self.set(account, new_balance);
    }
}
compound_key_value_storage!(Cep18AllowancesStorage, ALLOWANCES_KEY, Address, U256);
impl Cep18AllowancesStorage {
    /// Adds the given amount to the allowance of the given owner and spender.
    pub fn add(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), ExecutionError::AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }

    /// Subtracts the given amount from the allowance of the given owner and spender.
    pub fn subtract(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), ExecutionError::AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }
}
