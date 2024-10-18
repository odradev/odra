use base64::prelude::BASE64_STANDARD;
use base64::Engine;

use odra::casper_types::bytesrepr::ToBytes;
use odra::casper_types::U256;
use odra::prelude::*;

const VALUE_KEY: &str = "value";
const DICT_KEY: &str = "dict";

#[odra::module]
/// Storage module for the named key.
pub struct NamedKeyStorage;

#[odra::module]
impl NamedKeyStorage {
    /// Sets the value.
    pub fn set(&self, value: String) {
        self.env().set_named_value(VALUE_KEY, value);
    }

    /// Gets the value.
    pub fn get(&self) -> String {
        self.env()
            .get_named_value(VALUE_KEY)
            .unwrap_or_revert_with(&self.env(), ExecutionError::UnwrapError)
    }
}

#[odra::module]
/// Storage module for the dictionary value.
pub struct DictionaryStorage;

#[odra::module]
impl DictionaryStorage {
    /// Sets the value.
    pub fn set(&self, key: String, value: U256) {
        self.env()
            .set_dictionary_value(DICT_KEY, self.key(key).as_bytes(), value);
    }

    /// Gets the value.
    pub fn get_or_default(&self, key: String) -> U256 {
        self.env()
            .get_dictionary_value(DICT_KEY, self.key(key).as_bytes())
            .unwrap_or_default()
    }

    fn key(&self, key: String) -> String {
        BASE64_STANDARD.encode(self.env().hash(key.to_bytes().unwrap()))
    }
}
