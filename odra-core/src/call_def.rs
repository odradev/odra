use casper_types::bytesrepr::{FromBytes, ToBytes};
use odra::types::Balance;

#[derive(Clone)]
pub struct CallDef {
    pub method: String,
    pub args: Args,
    pub amount: Option<Balance>,
}

impl CallDef {
    pub fn new(method: String, args: Args, attached_value: Option<Balance>) -> Self {
        CallDef {
            method,
            args,
            amount: attached_value,
        }
    }

    pub fn args<T: FromBytes>(&self) -> T {
        T::from_bytes(&self.args.0).unwrap().0
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn attached_value(&self) -> Option<Balance> {
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
