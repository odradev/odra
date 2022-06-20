use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use odra_types::{Address, CLValue};

#[derive(Default, Clone)]
pub struct Storage(HashMap<u64, CLValue>);

impl Storage {
    pub fn insert_single_value(&mut self, ident: &Address, key: &[u8], value: CLValue) {
        let hash = Storage::hashed_key(ident, key);
        self.0.insert(hash, value);
    }

    pub fn insert_dict_value(
        &mut self,
        ident: &Address,
        collection: &[u8],
        key: &[u8],
        value: CLValue,
    ) {
        let dict_key = [collection, key].concat();
        let hash = Storage::hashed_key(ident, dict_key);
        self.0.insert(hash, value);
    }

    pub fn get(&self, ident: &Address, key: &[u8]) -> Option<CLValue> {
        let hash = Storage::hashed_key(ident, key);
        self.0.get(&hash).cloned()
    }

    pub fn get_dict_value(
        &self,
        ident: &Address,
        collection: &[u8],
        key: &[u8],
    ) -> Option<CLValue> {
        let dict_key = [collection, key].concat();
        let hash = Storage::hashed_key(ident, dict_key);
        self.0.get(&hash).cloned()
    }

    fn hashed_key<H: Hash>(ident: &Address, key: H) -> u64 {
        let mut hasher = DefaultHasher::new();
        ident.hash(&mut hasher);
        key.hash(&mut hasher);
        hasher.finish()
    }
}
