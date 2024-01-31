use crate::casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped
};
use crate::contract_env::ContractEnv;
use crate::module::{ModuleComponent, ModulePrimitive};
use crate::prelude::*;
use crate::{OdraError, UnwrapOrRevert};

/// Data structure for storing a single value.
pub struct Variable<T> {
    env: Rc<ContractEnv>,
    phantom: core::marker::PhantomData<T>,
    index: u8
}

impl<T> Variable<T> {
    /// Returns the contract environment associated with the variable.
    pub fn env(&self) -> ContractEnv {
        self.env.child(self.index)
    }
}

/// Implements the `ModuleComponent` trait for the `Variable` struct.
impl<T> ModuleComponent for Variable<T> {
    /// Creates a new instance of `Variable` with the given environment and index.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env,
            phantom: core::marker::PhantomData,
            index
        }
    }
}

impl<T> ModulePrimitive for Variable<T> {}

impl<T: FromBytes> Variable<T> {
    /// Retrieves the value of the variable.
    ///
    /// Returns `Some(value)` if the variable has a value, or `None` if it is unset.
    pub fn get(&self) -> Option<T> {
        let env = self.env();
        env.get_value(&env.current_key())
    }

    /// Retrieves the value of the variable or reverts with an error.
    ///
    /// If the variable has a value, it is returned. Otherwise, the provided error is reverted.
    pub fn get_or_revert_with<E: Into<OdraError>>(&self, error: E) -> T {
        let env = self.env();
        self.get().unwrap_or_revert_with(&env, error)
    }
}

impl<T: FromBytes + Default> Variable<T> {
    /// Returns the value of the variable, or the default value of the type if the variable is None.
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<T: ToBytes + CLTyped> Variable<T> {
    /// Sets the value of the variable.
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
