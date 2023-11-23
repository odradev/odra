use crate::{
    module::{Module, ModuleWrapper},
    prelude::*,
    variable::Variable,
    ContractEnv
};
use crate::{CLTyped, FromBytes, ToBytes};

pub struct Mapping<K, V> {
    parent_env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<(K, V)>,
    index: u8
}

impl<K: ToBytes, V> Mapping<K, V> {
    pub const fn new(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            parent_env: env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<K: ToBytes, V> Mapping<K, V> {
    fn env_for_key(&self, key: K) -> ContractEnv {
        let mut env = self.parent_env.duplicate();
        let key = key.to_bytes().unwrap_or_default();
        env.add_to_mapping_data(&key);
        env
    }
}

impl<K: ToBytes, V: FromBytes + CLTyped + Default> Mapping<K, V> {
    pub fn get_or_default(&self, key: K) -> V {
        let env = self.env_for_key(key);
        Variable::<V>::new(Rc::new(env), self.index).get_or_default()
    }
}

impl<K: ToBytes, V: ToBytes + CLTyped> Mapping<K, V> {
    pub fn set(&mut self, key: K, value: V) {
        let env = self.env_for_key(key);
        Variable::<V>::new(Rc::new(env), self.index).set(value)
    }
}

impl<K: ToBytes, V: Module> Mapping<K, V> {
    pub fn module(&self, key: K) -> ModuleWrapper<V> {
        let env = self.env_for_key(key);
        ModuleWrapper::new(Rc::new(env), self.index)
    }
}
