use alloc::string::String;
use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, RuntimeArgs, U512};

#[derive(Clone, Debug)]
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

    pub fn get<T: CLTyped + FromBytes>(&self, name: &str) -> Option<T> {
        self.args
            .get(name)
            .map(|v| v.clone().into_t().ok())
            .flatten()
    }

    pub fn method(&self) -> &str {
        &self.entry_point
    }

    pub fn attached_value(&self) -> Option<U512> {
        self.amount
    }
}
