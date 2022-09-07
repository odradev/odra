use anyhow::{Context, Result};
use odra_types::{Address, U256};

#[derive(Clone)]
pub struct Account {
    address: Address,
    native_token_balance: U256,
}

impl Account {
    pub fn new(address: Address, balance: U256) -> Self {
        Self {
            address,
            native_token_balance: balance,
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn balance(&self) -> U256 {
        self.native_token_balance
    }

    pub fn increase_balance(&mut self, amount: U256) -> Result<()> {
        let result = self
            .native_token_balance
            .checked_add(amount)
            .context("Addition overflow")?;
        self.native_token_balance = result;
        Ok(())
    }

    pub fn reduce_balance(&mut self, amount: U256) -> Result<()> {
        let result = self
            .native_token_balance
            .checked_sub(amount)
            .context("Subtraction overflow")?;
        self.native_token_balance = result;
        Ok(())
    }
}
