use std::mem::swap;

use anyhow::{Context, Result};
use odra_types::U512;

#[derive(Eq, Hash, PartialEq, Clone, Default)]
pub struct Balance {
    value: U512,
    prev_value: U512,
}

impl Balance {
    pub fn new(amount: U512) -> Self {
        Self {
            value: amount,
            prev_value: U512::zero(),
        }
    }

    pub fn increase(&mut self, amount: U512) -> Result<()> {
        let result = self
            .value
            .checked_add(amount)
            .context("Addition overflow")?;

        self.prev_value = self.value;
        self.value = result;
        Ok(())
    }

    pub fn reduce(&mut self, amount: U512) -> Result<()> {
        let result = self
            .value
            .checked_sub(amount)
            .context("Subtraction overflow")?;
        self.prev_value = self.value;
        self.value = result;
        Ok(())
    }

    pub fn revert(&mut self) {
        swap(&mut self.value, &mut self.prev_value);
    }

    pub fn value(&self) -> U512 {
        self.value
    }
}

impl From<u32> for Balance {
    fn from(value: u32) -> Self {
        Self::new(value.into())
    }
}
