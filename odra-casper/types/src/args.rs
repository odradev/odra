use std::ops::{Deref, DerefMut};

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};

#[derive(Default)]
pub struct CallArgs(casper_types::RuntimeArgs);

impl CallArgs {
    pub fn new() -> Self {
        Self(casper_types::RuntimeArgs::default())
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: CLTyped + ToBytes,
    {
        self.0.insert(key, value).unwrap();
    }
}

impl ToBytes for CallArgs {
    fn to_bytes(&self) -> Result<Vec<u8>, casper_types::bytesrepr::Error> {
        self.0.to_bytes()
    }

    fn serialized_length(&self) -> usize {
        self.0.serialized_length()
    }
}

impl FromBytes for CallArgs {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), casper_types::bytesrepr::Error> {
        casper_types::RuntimeArgs::from_bytes(bytes)
            .map(|(args, leftovers)| (CallArgs(args), leftovers))
    }
}

impl Deref for CallArgs {
    type Target = casper_types::RuntimeArgs;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CallArgs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
