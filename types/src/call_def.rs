use alloc::string::String;
use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, RuntimeArgs, U512};

#[derive(Clone, Debug)]
pub struct CallDef {
    pub entry_point: String,
    pub args: RuntimeArgs,
    pub amount: U512
}

impl CallDef {
    pub fn new(method: String, args: RuntimeArgs) -> Self {
        CallDef {
            entry_point: method,
            args,
            amount: U512::zero()
        }
    }

    pub fn with_amount(mut self, amount: U512) -> Self {
        self.amount = amount;
        self
    }

    pub fn args(&self) -> &RuntimeArgs {
        &self.args
    }

    pub fn get<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        self.args
            .get(name)
            .and_then(|v| v.clone().into_t().ok())
    }

    pub fn method(&self) -> &str {
        &self.entry_point
    }

    pub fn attached_value(&self) -> U512 {
        self.amount
    }
}
