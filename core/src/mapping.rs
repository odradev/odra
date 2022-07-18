use std::{
    hash::Hash,
    marker::PhantomData,
    ops::{Add, Sub}, fmt::Debug,
};

use crate::ContractEnv;
use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};
use crate::UnwrapOrRevert;

use crate::instance::Instance;

#[derive(Debug)]
pub struct Mapping<K, V> {
    name: String,
    key_ty: PhantomData<K>,
    value_ty: PhantomData<V>,
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Mapping<K, V> {
    pub fn new(name: String) -> Self {
        Mapping {
            name,
            key_ty: PhantomData::<K>::default(),
            value_ty: PhantomData::<V>::default(),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let result = ContractEnv::get_dict_value(&self.name, key);

        match result {
            Some(value) => Some(value.into_t::<V>().unwrap_or_revert()),
            None => None,
        }
    }

    pub fn set(&self, key: &K, value: V) {
        ContractEnv::set_dict_value(&self.name, key, value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + Default> Mapping<K, V> {
    pub fn get_or_default(&self, key: &K) -> V {
        self.get(key).unwrap_or_default()
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + Add<Output = V> + Default>
    Mapping<K, V>
{
    pub fn add(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        // TODO: check overflow
        let new_value = current_value + value;
        ContractEnv::set_dict_value(&self.name, key, new_value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + Sub<Output = V> + Default + Debug + PartialOrd>
    Mapping<K, V>
{
    pub fn subtract(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        // TODO: check overflow
        if value <= current_value {
            let new_value = current_value - value;
            ContractEnv::set_dict_value(&self.name, key, new_value);
        }
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> From<&str> for Mapping<K, V> {
    fn from(name: &str) -> Self {
        Mapping::new(name.to_string())
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Instance for Mapping<K, V> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}
