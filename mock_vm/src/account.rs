use std::mem::swap;

use anyhow::{Context, Result};
use odra_types::{Address, U512};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Account {
    address: Address,
    native_token_balance: U512,
    prev_native_token_balance: U512,
}

impl Account {
    pub fn new(address: Address, balance: U512) -> Self {
        Self {
            address,
            native_token_balance: balance,
            prev_native_token_balance: U512::zero(),
        }
    }

    pub fn zero_balance(address: Address) -> Self {
        Self {
            address,
            native_token_balance: U512::zero(),
            prev_native_token_balance: U512::zero(),
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn balance(&self) -> U512 {
        self.native_token_balance
    }

    pub fn increase_balance(&mut self, amount: U512) -> Result<()> {
        let result = self
            .native_token_balance
            .checked_add(amount)
            .context("Addition overflow")?;

        self.prev_native_token_balance = self.native_token_balance;
        self.native_token_balance = result;
        Ok(())
    }

    pub fn reduce_balance(&mut self, amount: U512) -> Result<()> {
        let result = self
            .native_token_balance
            .checked_sub(amount)
            .context("Subtraction overflow")?;
        self.prev_native_token_balance = self.native_token_balance;
        self.native_token_balance = result;
        Ok(())
    }

    pub fn revert_balance(&mut self) {
        swap(
            &mut self.native_token_balance,
            &mut self.prev_native_token_balance,
        );
    }
}
