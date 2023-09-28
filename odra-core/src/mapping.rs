use crate::{
    module::{Module, ModuleWrapper},
    prelude::*,
    variable::Variable,
    ContractEnv
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

pub struct Mapping<const N: u8, K, V> {
    parent_env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<(K, V)>
}

impl<const N: u8, K: ToBytes, V> Mapping<N, K, V> {
    pub const fn new(env: Rc<ContractEnv>) -> Self {
        Self {
            parent_env: env,
            phantom: core::marker::PhantomData
        }
    }
}

impl<const N: u8, K: ToBytes, V> Mapping<N, K, V> {
    fn env_for_key(&self, key: K) -> ContractEnv {
        let mut env = self.parent_env.duplicate();
        let key = key.to_bytes().unwrap_or_default();
        env.add_to_mapping_data(&key);
        env
    }
}

impl<const N: u8, K: ToBytes, V: FromBytes + CLTyped + Default> Mapping<N, K, V> {
    pub fn get_or_default(&self, key: K) -> V {
        let env = self.env_for_key(key);
        Variable::<N, V>::new(Rc::new(env)).get_or_default()
    }
}

impl<const N: u8, K: ToBytes, V: ToBytes + CLTyped> Mapping<N, K, V> {
    pub fn set(&mut self, key: K, value: V) {
        let env = self.env_for_key(key);
        Variable::<N, V>::new(Rc::new(env)).set(value)
    }
}

impl<const N: u8, K: ToBytes, V: Module> Mapping<N, K, V> {
    pub fn module(&self, key: K) -> ModuleWrapper<N, V> {
        let env = self.env_for_key(key);
        ModuleWrapper::<N, V>::new(Rc::new(env))
    }
}
