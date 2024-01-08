use crate::prelude::*;
use crate::{CLTyped, FromBytes, OdraError, ToBytes, UnwrapOrRevert};

use crate::contract_env::ContractEnv;
use crate::module::ModuleComponent;

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

impl<T> ModuleComponent for Variable<T> {
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<T: FromBytes> Variable<T> {
    pub fn get(&self) -> Option<T> {
        let env = self.env();
        env.get_value(&env.current_key())
    }

    pub fn get_or_revert_with<E: Into<OdraError>>(&self, error: E) -> T {
        let env = self.env();
        self.get().unwrap_or_revert_with(&env, error)
    }
}

impl<T: FromBytes + Default> Variable<T> {
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<T: ToBytes + CLTyped> Variable<T> {
    pub fn set(&mut self, value: T) {
        let env = self.env();
        env.set_value(&env.current_key(), value);
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Variable<V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    #[inline(always)]
    pub fn add(&mut self, value: V) {
        let current_value = self.get_or_default();
        let new_value = current_value
            .overflowing_add(value)
            .unwrap_or_revert(&self.env());
        self.set(new_value);
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default> Variable<V> {
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    #[inline(always)]
    pub fn subtract(&mut self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value
            .overflowing_sub(value)
            .unwrap_or_revert(&self.env());
        self.set(new_value);
    }
}
