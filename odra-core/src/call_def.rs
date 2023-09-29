use crate::prelude::*;
use casper_types::{RuntimeArgs, U512};

#[derive(Clone)]
pub struct CallDef {
    pub entry_point: String,
    pub args: RuntimeArgs,
    pub amount: Option<U512>
}

impl CallDef {
    pub fn new(method: String, args: RuntimeArgs, attached_value: Option<U512>) -> Self {
        CallDef {
            entry_point: method,
            args,
            amount: attached_value
        }
    }

    pub fn args(&self) -> &RuntimeArgs {
        &self.args
    }

    pub fn method(&self) -> &str {
        &self.entry_point
    }

    pub fn attached_value(&self) -> Option<U512> {
        self.amount
    }
}