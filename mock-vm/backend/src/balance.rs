use anyhow::{Context, Result};
use odra_mock_vm_types::Balance;

#[derive(Eq, Hash, PartialEq, Clone, Default, Debug)]
pub struct AccountBalance {
    value: Balance,
    prev_value: Balance
}

impl AccountBalance {
    pub fn new(amount: Balance) -> Self {
        Self {
            value: amount,
            prev_value: Balance::zero()
        }
    }

    pub fn increase(&mut self, amount: Balance) -> Result<()> {
        let result = self
            .value
            .checked_add(amount)
            .context("Addition overflow")?;

        self.prev_value = self.value;
        self.value = result;
        Ok(())
    }

    pub fn reduce(&mut self, amount: Balance) -> Result<()> {
        let result = self
            .value
            .checked_sub(amount)
            .context("Subtraction overflow")?;
        self.prev_value = self.value;
        self.value = result;
        Ok(())
    }

    pub fn value(&self) -> Balance {
        self.value
    }
}

impl From<u32> for AccountBalance {
    fn from(value: u32) -> Self {
        Self::new(value.into())
    }
}
