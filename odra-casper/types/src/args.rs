use std::ops::{Deref, DerefMut};

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

/// Represents a collection of arguments passed to a smart contract entrypoint call.
///
/// Wraps casper's [RuntimeArgs](casper_types::RuntimeArgs).
#[derive(Default)]
pub struct CallArgs(casper_types::RuntimeArgs);

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self(casper_types::RuntimeArgs::default())
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: CLTyped + ToBytes
    {
        self.0.insert(key, value).unwrap();
    }

    /// Retrieves a vector of argument names.
    pub fn arg_names(&self) -> Vec<String> {
        self.0
            .named_args()
            .map(|arg| arg.name().to_string())
            .collect()
    }

    /// Return Casper's RuntimeArgs.
    pub fn as_casper_runtime_args(&self) -> &casper_types::RuntimeArgs {
        &self.0
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
