use alloc::string::String;
use odra::casper_types::U256;
use odra::ExecutionError::AdditionOverflow;

use odra::casper_types::bytesrepr::ToBytes;
use odra::prelude::*;
use odra::{Address, UnwrapOrRevert};

use crate::cep18::errors::Error::{InvalidState, Overflow};

use base64::prelude::*;

const ALLOWANCES_KEY: &str = "allowances";
const BALANCES_KEY: &str = "balances";
const NAME_KEY: &str = "name";
const DECIMALS_KEY: &str = "decimals";
const SYMBOL_KEY: &str = "symbol";
const TOTAL_SUPPLY_KEY: &str = "total_supply";

#[odra::module]
/// Storage module for the name of the token.
pub struct Cep18NameStorage;

#[odra::module]
impl Cep18NameStorage {
    /// Sets the name of the token.
    pub fn set(&self, name: String) {
        self.env().set_named_value(NAME_KEY, name);
    }

    /// Gets the name of the token.
    pub fn get(&self) -> String {
        self.env()
            .get_named_value(NAME_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the number of decimals of the token.
pub struct Cep18DecimalsStorage;

#[odra::module]
impl Cep18DecimalsStorage {
    /// Sets the number of decimals of the token.
    pub fn set(&self, decimals: u8) {
        self.env().set_named_value(DECIMALS_KEY, decimals);
    }

    /// Gets the number of decimals of the token.
    pub fn get(&self) -> u8 {
        self.env()
            .get_named_value(DECIMALS_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the symbol of the token.
pub struct Cep18SymbolStorage;

#[odra::module]
impl Cep18SymbolStorage {
    /// Sets the symbol of the token.
    pub fn set(&self, symbol: String) {
        self.env().set_named_value(SYMBOL_KEY, symbol);
    }

    /// Gets the symbol of the token.
    pub fn get(&self) -> String {
        self.env()
            .get_named_value(SYMBOL_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }
}

#[odra::module]
/// Storage module for the total supply of the token.
pub struct Cep18TotalSupplyStorage;

#[odra::module]
impl Cep18TotalSupplyStorage {
    /// Sets the total supply of the token.
    pub fn set(&self, total_supply: U256) {
        self.env().set_named_value(TOTAL_SUPPLY_KEY, total_supply);
    }

    /// Gets the total supply of the token.
    pub fn get(&self) -> U256 {
        self.env()
            .get_named_value(TOTAL_SUPPLY_KEY)
            .unwrap_or_revert_with(&self.env(), InvalidState)
    }

    /// Adds the given amount to the total supply of the token.
    pub fn add(&self, amount: U256) {
        let total_supply = self.get();
        let new_total_supply = total_supply
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
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

#[odra::module]
/// Storage module for the balances of the token.
pub struct Cep18BalancesStorage;

#[odra::module]
impl Cep18BalancesStorage {
    /// Sets the balance of the given account.
    pub fn set(&self, account: &Address, balance: U256) {
        self.env()
            .set_dictionary_value(BALANCES_KEY, self.key(account).as_bytes(), balance);
    }

    /// Gets the balance of the given account.
    pub fn get_or_default(&self, account: &Address) -> U256 {
        self.env()
            .get_dictionary_value(BALANCES_KEY, self.key(account).as_bytes())
            .unwrap_or_default()
    }

    /// Adds the given amount to the balance of the given account.
    pub fn add(&self, account: &Address, amount: U256) {
        let balance = self.get_or_default(account);
        let new_balance = balance.checked_add(amount).unwrap_or_revert(&self.env());
        self.set(account, new_balance);
    }

    /// Subtracts the given amount from the balance of the given account.
    pub fn subtract(&self, account: &Address, amount: U256) {
        let balance = self.get_or_default(account);
        let new_balance = balance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), Overflow);
        self.set(account, new_balance);
    }

    fn key(&self, owner: &Address) -> String {
        // PRENOTE: This note is copied from the original implementation of CEP-18.
        // NOTE: As for now dictionary item keys are limited to 64 characters only. Instead of using
        // hashing (which will effectively hash a hash) we'll use base64. Preimage is 33 bytes for
        // both used Key variants, and approximated base64-encoded length will be 4 * (33 / 3) ~ 44
        // characters.
        // Even if the preimage increased in size we still have extra space but even in case of much
        // larger preimage we can switch to base85 which has ratio of 4:5.
        let preimage = owner.to_bytes().unwrap_or_revert(&self.env());
        BASE64_STANDARD.encode(preimage)
    }
}

#[odra::module]
/// Storage module for the allowances of the token.
pub struct Cep18AllowancesStorage;

#[odra::module]
impl Cep18AllowancesStorage {
    /// Sets the allowance of the given owner and spender.
    pub fn set(&self, owner: &Address, spender: &Address, amount: U256) {
        self.env()
            .set_dictionary_value(ALLOWANCES_KEY, &self.key(owner, spender), amount);
    }

    /// Gets the allowance of the given owner and spender.
    pub fn get_or_default(&self, owner: &Address, spender: &Address) -> U256 {
        self.env()
            .get_dictionary_value(ALLOWANCES_KEY, &self.key(owner, spender))
            .unwrap_or_default()
    }

    /// Adds the given amount to the allowance of the given owner and spender.
    pub fn add(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_add(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }

    /// Subtracts the given amount from the allowance of the given owner and spender.
    pub fn subtract(&self, owner: &Address, spender: &Address, amount: U256) {
        let allowance = self.get_or_default(owner, spender);
        let new_allowance = allowance
            .checked_sub(amount)
            .unwrap_or_revert_with(&self.env(), AdditionOverflow);
        self.set(owner, spender, new_allowance);
    }

    fn key(&self, owner: &Address, spender: &Address) -> [u8; 64] {
        let mut result = [0u8; 64];
        let mut preimage = Vec::new();
        preimage.append(&mut owner.to_bytes().unwrap_or_revert(&self.env()));
        preimage.append(&mut spender.to_bytes().unwrap_or_revert(&self.env()));

        let key_bytes = self.env().hash(&preimage);
        odra::utils::hex_to_slice(&key_bytes, &mut result);
        result
    }
}
