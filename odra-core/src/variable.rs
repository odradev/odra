use crate::prelude::*;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};

use crate::contract_env::ContractEnv;

pub struct Variable<const N: u8, T> {
    env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<T>
}

impl<const N: u8, T> Variable<N, T> {
    pub fn env(&self) -> ContractEnv {
        self.env.child(N)
    }
}

impl<const N: u8, T> Variable<N, T> {
    pub const fn new(env: Rc<ContractEnv>) -> Self {
        Self {
            env,
            phantom: core::marker::PhantomData
        }
    }
}

impl<const N: u8, T: FromBytes + Default> Variable<N, T> {
    pub fn get_or_default(&self) -> T {
        let env = self.env();
        env.get_value(&env.current_key()).unwrap_or_default()
    }
}

impl<const N: u8, T: ToBytes + CLTyped> Variable<N, T> {
    pub fn set(&mut self, value: T) {
        let env = self.env();
        env.set_value(&env.current_key(), value);
    }
}
