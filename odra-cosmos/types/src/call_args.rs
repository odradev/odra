use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::{CosmosType, Typed};

/// Represents a collection of arguments passed to a smart contract entrypoint call.
#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, Vec<u8>>
}

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K: Into<String>, V: CosmosType + Typed>(&mut self, key: K, value: V) {
        self.data.insert(key.into(), value.ser().unwrap());
    }

    /// Gets an argument by the name an returns as a [`serde_json::Value`].
    pub fn get_as_value(&self, key: &str) -> Value {
        let value = self.get_raw_bytes(key);
        Value::Array(value.iter().map(|v| json!(v)).collect())
    }

    /// Retrieves a vector of argument names.
    pub fn arg_names(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Gets an argument by the name.
    fn get_raw_bytes(&self, key: &str) -> Vec<u8> {
        self.data.get(key).cloned().unwrap_or_default()
    }
}
