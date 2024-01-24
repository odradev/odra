use crate::prelude::*;
use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, RuntimeArgs, U512};

#[derive(Clone, Debug)]
pub struct CallDef {
    pub entry_point: String,
    pub args: RuntimeArgs,
    pub amount: U512,
    is_mut: bool
}

impl CallDef {
    pub fn new<T: ToString>(method: T, is_mut: bool, args: RuntimeArgs) -> Self {
        CallDef {
            entry_point: method.to_string(),
            args,
            amount: U512::zero(),
            is_mut
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
        self.args.get(name).and_then(|v| v.clone().into_t().ok())
    }

    pub fn method(&self) -> &str {
        &self.entry_point
    }

    pub fn attached_value(&self) -> U512 {
        self.amount
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }
}
