use crate::{CosmosType, Typed};

/// Represents a collection of arguments passed to a smart contract entrypoint call.
#[derive(Default, Debug)]
pub struct CallArgs {
    data: Vec<Vec<u8>>,
    keys: Vec<String>,
}

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K: Into<String>, V: CosmosType + Typed>(&mut self, key: K, value: V) {
        self.data.push(value.ser().unwrap());
        self.keys.push(key.into());
    }

    /// Gets an argument by the name an returns as a [`serde_json::Value`].
    pub fn get_as_json(&self, key: &str) -> String {
        let value = self.get_raw_bytes(key);
        dbg!(key);
        dbg!(&value);
        serde_json_wasm::to_string(&value).unwrap()
    }

    /// Retrieves a vector of argument names.
    pub fn arg_names(&self) -> Vec<String> {
        self.keys.clone()
    }

    /// Gets an argument by the name.
    pub fn get_raw_bytes(&self, key: &str) -> Vec<u8> {
        let idx = self.keys.iter().position(|k| k == key).unwrap();

        self.data.get(idx).cloned().unwrap_or_default()
    }
}
