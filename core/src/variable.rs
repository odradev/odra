use crate::prelude::*;
use crate::{CLTyped, FromBytes, OdraError, ToBytes, UnwrapOrRevert};

use crate::contract_env::ContractEnv;

pub struct Variable<T> {
    env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<T>,
    index: u8
}

impl<T> Variable<T> {
    pub fn env(&self) -> ContractEnv {
        self.env.child(self.index)
    }
}

impl<T> Variable<T> {
    pub const fn new(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<T: FromBytes + Default> Variable<T> {
    pub fn get_or_default(&self) -> T {
        let env = self.env();
        env.get_value(&env.current_key()).unwrap_or_default()
    }

    pub fn get(&self) -> Option<T> {
        let env = self.env();
        env.get_value(&env.current_key())
    }

    pub fn get_or_revert_with<E: Into<OdraError>>(&self, error: E) -> T {
        let env = self.env();
        env.get_value(&env.current_key())
            .unwrap_or_revert_with(&env, error)
    }
}

impl<T: ToBytes + CLTyped> Variable<T> {
    pub fn set(&mut self, value: T) {
        let env = self.env();
        env.set_value(&env.current_key(), value);
    }
}

impl<T> crate::contract_def::HasEvents for Variable<T> {
    fn events() -> Vec<crate::contract_def::Event> {
        vec![]
    }
}
