use casper_types::bytesrepr::FromBytes;
use casper_types::{CLTyped, CLValue, RuntimeArgs};

pub trait UncheckedGetter {
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T;
}

impl UncheckedGetter for RuntimeArgs {
    fn get<T: FromBytes + CLTyped>(&self, key: &str) -> T {
        self.get(key)
            .cloned()
            .map(CLValue::into_t)
            .unwrap()
            .unwrap()
    }
}
