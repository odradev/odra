use crate::arithmetic::{OverflowingAdd, OverflowingSub};
use crate::casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};
use crate::module::{ModuleComponent, ModulePrimitive, Revertible};
use crate::{
    module::{Module, SubModule},
    var::Var,
    ContractEnv, UnwrapOrRevert
};
use crate::{prelude::*, OdraError};
use core::fmt::Debug;

/// Data structure for storing key-value pairs.
pub struct Mapping<K, V> {
    parent_env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<(K, V)>,
    index: u8
}

impl<K: ToBytes, V> ModuleComponent for Mapping<K, V> {
    /// Creates a new instance of `Mapping` with the given environment and index.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            parent_env: env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<K: ToBytes, V> Revertible for Mapping<K, V> {
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.parent_env.revert(e)
    }
}

impl<K: ToBytes, V> ModulePrimitive for Mapping<K, V> {}

impl<K: ToBytes, V> Mapping<K, V> {
    fn env_for_key(&self, key: &K) -> ContractEnv {
        let mut env = (*self.parent_env).clone();
        let key = key.to_bytes().unwrap_or_default();
        env.add_to_mapping_data(&key);
        env
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped> Mapping<K, V> {
    /// Retrieves the value associated with the given key.
    ///
    /// Returns an `Option<V>` representing the value associated with the key, or `None` if the key is not found.
    pub fn get(&self, key: &K) -> Option<V> {
        let env = self.env_for_key(key);
        Var::<V>::instance(Rc::new(env), self.index).get()
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped + Default> Mapping<K, V> {
    /// Retrieves the value associated with the given key from the mapping.
    /// If the key does not exist, returns the default value of type `V`.
    pub fn get_or_default(&self, key: &K) -> V {
        let env = self.env_for_key(key);
        Var::<V>::instance(Rc::new(env), self.index).get_or_default()
    }
}

impl<K: ToBytes, V: ToBytes + CLTyped> Mapping<K, V> {
    /// Sets the value associated with the given key in the mapping.
    pub fn set(&mut self, key: &K, value: V) {
        let env = self.env_for_key(key);
        Var::<V>::instance(Rc::new(env), self.index).set(value)
    }
}

impl<K: ToBytes, V: Module> Mapping<K, V> {
    /// Retrieves the module associated with the given key.
    ///
    /// A [`SubModule`] instance containing the module associated with the key.
    pub fn module(&self, key: &K) -> SubModule<V> {
        let env = self.env_for_key(key);
        SubModule::instance(Rc::new(env), self.index)
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Mapping<K, V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, key: &K, value: V) {
        let env = self.env_for_key(key);
        let current_value = Var::<V>::instance(Rc::new(env.clone()), self.index).get_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert(self);
        Var::<V>::instance(Rc::new(env), self.index).set(new_value);
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
        let env = self.env_for_key(key);
        let current_value = Var::<V>::instance(Rc::new(env.clone()), self.index).get_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert(self);
        Var::<V>::instance(Rc::new(env), self.index).set(new_value);
    }
}
