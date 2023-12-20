use crate::arithmetic::{OverflowingAdd, OverflowingSub};
use crate::module::ModuleComponent;
use crate::prelude::*;
use crate::{
    module::{Module, ModuleWrapper},
    variable::Variable,
    ContractEnv, UnwrapOrRevert
};
use crate::{CLTyped, FromBytes, ToBytes};
use core::fmt::Debug;

pub struct Mapping<K, V> {
    parent_env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<(K, V)>,
    index: u8
}

impl<K: ToBytes, V> ModuleComponent for Mapping<K, V> {
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            parent_env: env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<K: ToBytes, V> Mapping<K, V> {
    fn env_for_key(&self, key: &K) -> ContractEnv {
        let mut env = (*self.parent_env).clone();
        let key = key.to_bytes().unwrap_or_default();
        env.add_to_mapping_data(&key);
        env
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped + Default> Mapping<K, V> {
    pub fn get_or_default(&self, key: &K) -> V {
        let env = self.env_for_key(key);
        Variable::<V>::instance(Rc::new(env), self.index).get_or_default()
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped> Mapping<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        let env = self.env_for_key(key);
        Variable::<V>::instance(Rc::new(env), self.index).get()
    }
}

impl<K: ToBytes, V: ToBytes + CLTyped> Mapping<K, V> {
    pub fn set(&mut self, key: &K, value: V) {
        let env = self.env_for_key(key);
        Variable::<V>::instance(Rc::new(env), self.index).set(value)
    }
}

impl<K: ToBytes, V: Module> Mapping<K, V> {
    pub fn module(&self, key: &K) -> ModuleWrapper<V> {
        let env = self.env_for_key(key);
        ModuleWrapper::instance(Rc::new(env), self.index)
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Mapping<K, V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, key: &K, value: V) {
        let current_value = self.get_or_default(key);
        let new_value = current_value
            .overflowing_add(value)
            .unwrap_or_revert(&self.env_for_key(key));
        self.set(key, new_value);
    }
}

impl<
        K: ToBytes,
        V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default + Debug + PartialOrd
    > Mapping<K, V>
{
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&mut self, key: &K, value: V) {
        let current_value = self.get_or_default(key);
        let new_value = current_value
            .overflowing_sub(value)
            .unwrap_or_revert(&self.env_for_key(key));
        self.set(key, new_value);
    }
}
