use crate::arithmetic::{OverflowingAdd, OverflowingSub};
use crate::casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};
use crate::contract_env::StorageSlot;
use crate::module::{ModuleComponent, ModulePrimitive};
use crate::prelude::*;
use crate::ContractEnv;
use core::fmt::Debug;

/// Data structure for storing key-value pairs.
pub struct Mapping<K, V> {
    parent_env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<(K, V)>,
    slot: StorageSlot
}

impl<K: ToBytes, V> ModuleComponent for Mapping<K, V> {
    /// Creates a new instance of `Mapping` with the given environment and index.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self::new_with_slot(env, StorageSlot::user(index))
    }
}

impl<K: ToBytes, V> Mapping<K, V> {
    pub(crate) fn internal_instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self::new_with_slot(env, StorageSlot::internal(index))
    }

    fn new_with_slot(env: Rc<ContractEnv>, slot: StorageSlot) -> Self {
        Self {
            parent_env: env,
            phantom: core::marker::PhantomData,
            slot
        }
    }

    fn env_for_key(&self, key: &K) -> ContractEnv {
        let mut env = (*self.parent_env).clone();
        let key = key.to_bytes().unwrap_or_default();
        env.add_to_mapping_data(&key);
        env
    }

    fn var<T>(&self, env: ContractEnv) -> Var<T> {
        match self.slot {
            StorageSlot::User(index) => Var::instance(Rc::new(env), index),
            StorageSlot::Internal(index) => Var::internal_instance(Rc::new(env), index)
        }
    }

    fn submodule<T: Module>(&self, env: ContractEnv) -> SubModule<T> {
        match self.slot {
            StorageSlot::User(index) => SubModule::instance(Rc::new(env), index),
            StorageSlot::Internal(index) => SubModule::internal_instance(Rc::new(env), index)
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
}

impl<K: ToBytes, V: FromBytes + CLTyped> Mapping<K, V> {
    /// Retrieves the value associated with the given key.
    ///
    /// Returns an `Option<V>` representing the value associated with the key, or `None` if the key is not found.
    pub fn get(&self, key: &K) -> Option<V> {
        let env = self.env_for_key(key);
        self.var::<V>(env).get()
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped + Default> Mapping<K, V> {
    /// Retrieves the value associated with the given key from the mapping.
    /// If the key does not exist, returns the default value of type `V`.
    pub fn get_or_default(&self, key: &K) -> V {
        let env = self.env_for_key(key);
        self.var::<V>(env).get_or_default()
    }
}

impl<K: ToBytes, V: ToBytes + CLTyped> Mapping<K, V> {
    /// Sets the value associated with the given key in the mapping.
    pub fn set(&mut self, key: &K, value: V) {
        let env = self.env_for_key(key);
        self.var::<V>(env).set(value)
    }
}

impl<K: ToBytes, V: Module> Mapping<K, V> {
    /// Retrieves the module associated with the given key.
    ///
    /// A [`SubModule`] instance containing the module associated with the key.
    pub fn module(&self, key: &K) -> SubModule<V> {
        let env = self.env_for_key(key);
        self.submodule::<V>(env)
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Mapping<K, V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, key: &K, value: V) {
        let env = self.env_for_key(key);
        let mut var = self.var::<V>(env);
        var.add(value);
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
        let mut var = self.var::<V>(env);
        var.subtract(value);
    }
}
