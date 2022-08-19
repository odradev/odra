use core::hash::Hash;
use core::marker::PhantomData;

use crate::ContractEnv;
use crate::UnwrapOrRevert;
use alloc::string::String;
use odra_types::{
    arithmetic::{OverflowingAdd, OverflowingSub},
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};

use crate::instance::Instance;

/// Data structure for storing key-value pairs.
#[derive(Debug)]
pub struct Mapping<K, V> {
    name: String,
    key_ty: PhantomData<K>,
    value_ty: PhantomData<V>,
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Mapping<K, V> {
    /// Creates a new Mapping instance.
    pub fn new(name: String) -> Self {
        Mapping {
            name,
            key_ty: PhantomData::<K>::default(),
            value_ty: PhantomData::<V>::default(),
        }
    }

    /// Reads `key` from the storage or returns `None`.
    pub fn get(&self, key: &K) -> Option<V> {
        let result = ContractEnv::get_dict_value(&self.name, key);
        result.map(|value| value.into_t::<V>().unwrap_or_revert())
    }

    /// Sets `value` under `key` to the storage. It overrides by default.
    pub fn set(&self, key: &K, value: V) {
        ContractEnv::set_dict_value(&self.name, key, value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + Default> Mapping<K, V> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, key: &K) -> V {
        self.get(key).unwrap_or_default()
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default>
    Mapping<K, V>
{
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        ContractEnv::set_dict_value(&self.name, key, new_value);
    }
}

impl<
        K: ToBytes + CLTyped + Hash,
        V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default + PartialOrd,
    > Mapping<K, V>
{
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        ContractEnv::set_dict_value(&self.name, key, new_value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> From<&str> for Mapping<K, V> {
    fn from(name: &str) -> Self {
        Mapping::new(String::from(name))
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Instance for Mapping<K, V> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}
