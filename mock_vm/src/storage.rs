use anyhow::{Context, Result};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use odra_types::{Address, CLValue, U512};

use crate::balance::Balance;

#[derive(Default, Clone)]
pub struct Storage {
    state: HashMap<u64, CLValue>,
    balances: HashMap<Address, Balance>,
    state_snapshot: Option<HashMap<u64, CLValue>>,
    balances_snapshot: Option<HashMap<Address, Balance>>,
}

impl Storage {
    pub fn new(balances: HashMap<Address, Balance>) -> Self {
        Self {
            state: Default::default(),
            balances,
            state_snapshot: Default::default(),
            balances_snapshot: Default::default(),
        }
    }

    pub fn balance_of(&self, address: &Address) -> Option<&Balance> {
        self.balances.get(&address)
    }

    pub fn set_balance(&mut self, address: Address, balance: Balance) {
        self.balances.insert(address, balance);
    }

    pub fn increase_balance(&mut self, address: &Address, amount: U512) -> Result<()> {
        let balance = self.balances.get_mut(address).context("Unknown address")?;
        balance.increase(amount)
    }

    pub fn reduce_balance(&mut self, address: &Address, amount: U512) -> Result<()> {
        let balance = self.balances.get_mut(address).context("Unknown address")?;
        balance.reduce(amount)
    }

    pub fn get_value(&self, address: &Address, key: &[u8]) -> Option<CLValue> {
        let hash = Storage::hashed_key(address, key);
        self.state.get(&hash).cloned()
    }

    pub fn set_value(&mut self, address: &Address, key: &[u8], value: CLValue) {
        let hash = Storage::hashed_key(address, key);
        self.state.insert(hash, value);
    }

    pub fn insert_dict_value(
        &mut self,
        address: &Address,
        collection: &[u8],
        key: &[u8],
        value: CLValue,
    ) {
        let dict_key = [collection, key].concat();
        let hash = Storage::hashed_key(address, dict_key);
        self.state.insert(hash, value);
    }

    pub fn get_dict_value(
        &self,
        address: &Address,
        collection: &[u8],
        key: &[u8],
    ) -> Option<CLValue> {
        let dict_key = [collection, key].concat();
        let hash = Storage::hashed_key(address, dict_key);
        self.state.get(&hash).cloned()
    }

    pub fn take_snapshot(&mut self) {
        self.state_snapshot = Some(self.state.clone());
        self.balances_snapshot = Some(self.balances.clone());
    }

    pub fn drop_snapshot(&mut self) {
        self.state_snapshot = None;
        self.balances_snapshot = None;
    }

    pub fn restore_snapshot(&mut self) {
        if let Some(snapshot) = self.state_snapshot.clone() {
            self.state = snapshot;
            self.state_snapshot = None;
        };
        if let Some(snapshot) = self.balances_snapshot.clone() {
            self.balances = snapshot;
            self.balances_snapshot = None;
        };
    }

    fn hashed_key<H: Hash>(address: &Address, key: H) -> u64 {
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        key.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod test {
    use odra_types::{Address, CLValue};

    use super::Storage;

    fn setup() -> (Address, [u8; 2], CLValue) {
        let address = Address::new(b"address");
        let key = [1u8, 2u8];
        let value = CLValue::from_t(1_000u32).unwrap();

        (address, key, value)
    }

    #[test]
    fn read_write_single_value() {
        // given an empty storage
        let mut storage = Storage::default();
        let (address, key, value) = setup();

        // when put a value
        storage.set_value(&address, &key, value.clone());

        // then the value can be read
        assert_eq!(storage.get_value(&address, &key), Some(value));
    }

    #[test]
    fn override_single_value() {
        // given a storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        storage.set_value(&address, &key, value);

        // when the next value is set under the same key
        let next_value = CLValue::from_t(2_000u32).unwrap();
        storage.set_value(&address, &key, next_value.clone());

        // then the previous value is replaced
        assert_eq!(storage.get_value(&address, &key), Some(next_value));
    }

    #[test]
    fn read_non_existing_key_returns_none() {
        // given an empty storage
        let storage = Storage::default();
        let (address, key, _) = setup();

        // when lookup a key
        let result = storage.get_value(&address, &key);

        // then the None value is returned
        assert_eq!(result, None);
    }

    #[test]
    fn read_write_dict_value() {
        // given an empty storage
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = [1u8];

        // when put a value into a collection
        storage.insert_dict_value(&address, &collection, &key, value.clone());

        // then the value can be read
        assert_eq!(
            storage.get_dict_value(&address, &collection, &key),
            Some(value)
        );
    }

    #[test]
    fn read_from_non_existing_collection_returns_none() {
        // given storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = [1u8];
        storage.insert_dict_value(&address, &collection, &key, value);

        // when read a value from a non exisiting collection
        let non_existing_collection = [2u8];
        let result = storage.get_dict_value(&address, &non_existing_collection, &key);

        // then None is returned
        assert_eq!(result, None);
    }

    #[test]
    fn read_from_non_existing_key_from_existing_collection_returns_none() {
        // given storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = [1u8];
        storage.insert_dict_value(&address, &collection, &key, value);

        // when read a value from a non existing collection
        let non_existing_key = [2u8];
        let result = storage.get_dict_value(&address, &collection, &non_existing_key);

        // then None is returned
        assert_eq!(result, None);
    }

    #[test]
    fn restore_snapshot() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        storage.set_value(&address, &key, initial_value.clone());
        storage.take_snapshot();
        let next_value = CLValue::from_t(2_000u32).unwrap();
        storage.set_value(&address, &key, next_value);

        // when restore the snapshot
        storage.restore_snapshot();

        // then the changes are reverted
        assert_eq!(storage.get_value(&address, &key), Some(initial_value),);
        // the snapshot is removed
        assert_eq!(storage.state_snapshot, None);
    }

    #[test]
    fn test_snapshot_override() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        let second_value = CLValue::from_t(2_000u32).unwrap();
        let third_value = CLValue::from_t(3_000u32).unwrap();
        storage.set_value(&address, &key, initial_value);
        storage.take_snapshot();
        storage.set_value(&address, &key, second_value.clone());

        // when take another snapshot and restore it
        storage.take_snapshot();
        storage.set_value(&address, &key, third_value);
        storage.restore_snapshot();

        // then the most recent snapshot is restored
        assert_eq!(storage.get_value(&address, &key), Some(second_value),);
    }

    #[test]
    fn drop_snapshot() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        let next_value = CLValue::from_t(1_000u32).unwrap();
        storage.set_value(&address, &key, initial_value);
        storage.take_snapshot();
        storage.set_value(&address, &key, next_value.clone());

        // when the snapshot is dropped
        storage.drop_snapshot();

        // then storage state does not change
        assert_eq!(storage.get_value(&address, &key), Some(next_value),);
        // the snapshot is wiped out
        assert_eq!(storage.state_snapshot, None);
    }
}
