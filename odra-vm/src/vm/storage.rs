use anyhow::{Context, Result};
use odra_core::casper_types::{
    bytesrepr::{Bytes, Error, FromBytes, ToBytes},
    U512
};
use odra_core::prelude::*;
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher}
};

use super::balance::AccountBalance;

#[derive(Default, Clone)]
pub struct Storage {
    state: BTreeMap<u64, Bytes>,
    named_state: BTreeMap<u64, BTreeMap<u64, Bytes>>,
    pub balances: BTreeMap<Address, AccountBalance>,
    state_snapshot: Option<BTreeMap<u64, Bytes>>,
    named_state_snapshot: Option<BTreeMap<u64, BTreeMap<u64, Bytes>>>,
    balances_snapshot: Option<BTreeMap<Address, AccountBalance>>
}

impl Storage {
    pub fn new(balances: BTreeMap<Address, AccountBalance>) -> Self {
        Self {
            state: Default::default(),
            named_state: Default::default(),
            balances,
            state_snapshot: Default::default(),
            named_state_snapshot: Default::default(),
            balances_snapshot: Default::default()
        }
    }

    pub fn balance_of(&self, address: &Address) -> Option<&AccountBalance> {
        self.balances.get(address)
    }

    pub fn set_balance(&mut self, address: Address, balance: AccountBalance) {
        self.balances.insert(address, balance);
    }

    pub fn transfer(&mut self, from: &Address, to: &Address, amount: &U512) -> Result<()> {
        let from_balance = self.balances.get_mut(from).context("Unknown address")?;
        from_balance.reduce(*amount)?;
        let to_balance = self.balances.get_mut(to).context("Unknown address")?;
        to_balance.increase(*amount)
    }

    pub fn get_value(&self, address: &Address, key: &[u8]) -> Result<Option<Bytes>, Error> {
        let hash = Storage::hashed_key(address, key);
        let result = self.state.get(&hash).cloned();

        match result {
            Some(res) => Ok(Some(res)),
            None => Ok(None)
        }
    }

    pub fn set_value(&mut self, address: &Address, key: &[u8], value: Bytes) -> Result<(), Error> {
        let hash = Storage::hashed_key(address, key);
        self.state.insert(hash, value);
        Ok(())
    }

    pub fn insert_dict_value(
        &mut self,
        address: &Address,
        collection: &[u8],
        key: &[u8],
        value: Bytes
    ) -> Result<(), Error> {
        let dict = Self::hashed_key(address, collection);
        let hash = Storage::hashed_key(address, key);
        let dict_values = self.named_state.entry(dict).or_default();
        dict_values.insert(hash, value);
        Ok(())
    }

    pub fn remove_dict(&mut self, address: &Address, collection: &[u8]) {
        let dict = Self::hashed_key(address, collection);
        self.named_state.remove(&dict);
    }

    pub fn get_dict_value(
        &self,
        address: &Address,
        collection: &[u8],
        key: &[u8]
    ) -> Result<Option<Bytes>, Error> {
        let dict = Self::hashed_key(address, collection);
        let hash = Storage::hashed_key(address, key);
        let dict_values = self.named_state.get(&dict);
        if let Some(dict) = dict_values {
            let result = dict.get(&hash).cloned();
            Ok(result)
        } else {
            Ok(None)
        }
    }

    pub fn take_snapshot(&mut self) {
        self.state_snapshot = Some(self.state.clone());
        self.named_state_snapshot = Some(self.named_state.clone());
        self.balances_snapshot = Some(self.balances.clone());
    }

    pub fn drop_snapshot(&mut self) {
        self.state_snapshot = None;
        self.named_state_snapshot = None;
        self.balances_snapshot = None;
    }

    pub fn restore_snapshot(&mut self) {
        if let Some(snapshot) = self.state_snapshot.clone() {
            self.state = snapshot;
            self.state_snapshot = None;
        };
        if let Some(snapshot) = self.named_state_snapshot.clone() {
            self.named_state = snapshot;
            self.named_state_snapshot = None;
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
    use odra_core::casper_types::bytesrepr::Bytes;
    use odra_core::prelude::Address;
    use odra_core::utils::serialize;

    use crate::vm::utils;

    use super::Storage;

    fn setup() -> (Address, [u8; 3], u8) {
        let address = utils::account_address_from_str("add");
        let key = b"key";
        let value = 88u8;

        (address, *key, value)
    }

    #[test]
    fn read_write_single_value() {
        // given an empty storage
        let mut storage = Storage::default();
        let (address, key, value) = setup();

        // when put a value
        storage
            .set_value(&address, &key, serialize(&value))
            .unwrap();

        // then the value can be read
        assert_eq!(
            storage.get_value(&address, &key).unwrap(),
            Some(serialize(&value))
        );
    }

    #[test]
    fn override_single_value() {
        // given a storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        storage
            .set_value(&address, &key, serialize(&value))
            .unwrap();

        // when the next value is set under the same key
        let next_value = String::from("new_value");
        storage
            .set_value(&address, &key, serialize(&next_value))
            .unwrap();

        // then the previous value is replaced
        assert_eq!(
            storage.get_value(&address, &key).unwrap(),
            Some(serialize(&next_value))
        );
    }

    #[test]
    fn read_non_existing_key_returns_none() {
        // given an empty storage
        let storage = Storage::default();
        let (address, key, _) = setup();

        // when lookup a key
        let result: Option<Bytes> = storage.get_value(&address, &key).unwrap();

        // then the None value is returned
        assert_eq!(result, None);
    }

    #[test]
    fn read_write_dict_value() {
        // given an empty storage
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = b"dict";

        // when put a value into a collection
        storage
            .insert_dict_value(&address, collection, &key, serialize(&value))
            .unwrap();

        // then the value can be read
        assert_eq!(
            storage.get_dict_value(&address, collection, &key).unwrap(),
            Some(serialize(&value))
        );
    }

    #[test]
    fn read_from_non_existing_collection_returns_none() {
        // given storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = b"dict";
        storage
            .insert_dict_value(&address, collection, &key, serialize(&value))
            .unwrap();

        // when read a value from a non exisiting collection
        let non_existing_collection = b"collection";
        let result: Option<Bytes> = storage
            .get_dict_value(&address, non_existing_collection, &key)
            .unwrap();

        // then None is returned
        assert_eq!(result, None);
    }

    #[test]
    fn read_from_non_existing_key_from_existing_collection_returns_none() {
        // given storage with some stored value
        let mut storage = Storage::default();
        let (address, key, value) = setup();
        let collection = b"dict";
        storage
            .insert_dict_value(&address, collection, &key, serialize(&value))
            .unwrap();

        // when read a value from a non existing collection
        let non_existing_key = [2u8];
        let result: Option<Bytes> = storage
            .get_dict_value(&address, collection, &non_existing_key)
            .unwrap();

        // then None is returned
        assert_eq!(result, None);
    }

    #[test]
    fn remove_dict_erases_all_dict_records() {
        // given storage with some stored value
        let mut storage = Storage::default();
        let address = utils::account_address_from_str("add");
        let key1 = b"key";
        let key2 = b"key2";
        let value1 = 88u8;
        let value2 = 89u8;
        let collection = b"dict";
        storage
            .insert_dict_value(&address, collection, key1, serialize(&value1))
            .unwrap();
        storage
            .insert_dict_value(&address, collection, key2, serialize(&value2))
            .unwrap();

        assert_eq!(
            storage.get_dict_value(&address, collection, key1).unwrap(),
            Some(serialize(&value1))
        );
        assert_eq!(
            storage.get_dict_value(&address, collection, key2).unwrap(),
            Some(serialize(&value2))
        );

        // when remove a dictionary
        storage.remove_dict(&address, collection);

        // then all values from the dictionary are removed
        assert_eq!(
            storage.get_dict_value(&address, collection, key1).unwrap(),
            None
        );
        assert_eq!(
            storage.get_dict_value(&address, collection, key2).unwrap(),
            None
        );
    }

    #[test]
    fn restore_snapshot() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        storage
            .set_value(&address, &key, serialize(&initial_value))
            .unwrap();
        storage
            .insert_dict_value(&address, b"dict", &key, serialize(&initial_value))
            .unwrap();
        storage.take_snapshot();
        let next_value = String::from("next_value");
        storage
            .set_value(&address, &key, serialize(&next_value))
            .unwrap();
        storage
            .insert_dict_value(&address, b"dict", &key, serialize(&next_value))
            .unwrap();

        // when restore the snapshot
        storage.restore_snapshot();

        // then the changes are reverted
        assert_eq!(
            storage.get_value(&address, &key).unwrap(),
            Some(serialize(&initial_value))
        );
        assert_eq!(
            storage.get_dict_value(&address, b"dict", &key).unwrap(),
            Some(serialize(&initial_value))
        );
        // the snapshot is removed
        assert_eq!(storage.state_snapshot, None);
    }

    #[test]
    fn test_snapshot_override() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        let second_value = 2_000u32;
        let third_value = 3_000u32;
        storage
            .set_value(&address, &key, serialize(&initial_value))
            .unwrap();
        storage.take_snapshot();
        storage
            .set_value(&address, &key, serialize(&second_value))
            .unwrap();

        // when take another snapshot and restore it
        storage.take_snapshot();
        storage
            .set_value(&address, &key, serialize(&third_value))
            .unwrap();
        storage.restore_snapshot();

        // then the most recent snapshot is restored
        assert_eq!(
            storage.get_value(&address, &key).unwrap(),
            Some(serialize(&second_value)),
        );
    }

    #[test]
    fn drop_snapshot() {
        // given storage with some state and a snapshot of the previous state
        let mut storage = Storage::default();
        let (address, key, initial_value) = setup();
        let next_value = 1_000u32;
        storage
            .set_value(&address, &key, serialize(&initial_value))
            .unwrap();
        storage.take_snapshot();
        storage
            .set_value(&address, &key, serialize(&next_value))
            .unwrap();

        // when the snapshot is dropped
        storage.drop_snapshot();

        // then storage state does not change
        assert_eq!(
            storage.get_value(&address, &key).unwrap(),
            Some(serialize(&next_value)),
        );
        // the snapshot is wiped out
        assert_eq!(storage.state_snapshot, None);
    }
}
