use crate::prelude::*;
use casper_types::bytesrepr::{FromBytes, ToBytes};
use casper_types::U512;

#[derive(Clone)]
pub struct CallDef {
    pub entry_point: String,
    pub args: Args,
    pub amount: Option<U512>
}

impl CallDef {
    pub fn new(method: String, args: Args, attached_value: Option<U512>) -> Self {
        CallDef {
            entry_point: method,
            args,
            amount: attached_value
        }
    }

    pub fn args<T: FromBytes>(&self) -> T {
        T::from_bytes(&self.args.0).unwrap().0
    }

    pub fn method(&self) -> &str {
        &self.entry_point
    }

    pub fn attached_value(&self) -> Option<U512> {
        self.amount
    }
}

#[derive(Clone)]
pub struct Args(Vec<u8>);

impl<T: ToBytes> From<T> for Args {
    fn from(key: T) -> Self {
        Args(key.to_bytes().unwrap())
    }
}
