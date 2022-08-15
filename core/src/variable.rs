use core::marker::PhantomData;

use alloc::string::String;
use odra_types::{
    arithmetic::{OverflowingAdd, OverflowingSub},
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};

use crate::ContractEnv;
use crate::{instance::Instance, UnwrapOrRevert};

/// Data structure for storing a single value.
#[derive(PartialEq, Eq, Debug)]
pub struct Variable<T> {
    name: String,
    ty: PhantomData<T>,
}

// <3
impl<T: FromBytes + ToBytes + CLTyped + Default> Variable<T> {
    /// Reads from the storage, if theres no value in the storage the default value is returned.
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Variable<V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        ContractEnv::set_var(&self.name, new_value);
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default> Variable<V> {
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        ContractEnv::set_var(&self.name, new_value);
    }
}

impl<T: FromBytes + ToBytes + CLTyped> Variable<T> {
    /// Creates a new Variable instance.
    pub fn new(name: String) -> Self {
        Variable {
            name,
            ty: PhantomData::<T>::default(),
        }
    }

    /// Reads from the storage or returns `None` or reverts something unexpected happens.
    pub fn get(&self) -> Option<T> {
        ContractEnv::get_var(&self.name).map(|value| value.into_t::<T>().unwrap_or_revert())
    }

    /// Stores `value` to the storage.
    pub fn set(&self, value: T) {
        ContractEnv::set_var(&self.name, value);
    }

    /// Return the named key path to the variable.
    pub fn path(&self) -> &str {
        &self.name
    }
}

impl<T: FromBytes + ToBytes + CLTyped> From<&str> for Variable<T> {
    fn from(name: &str) -> Self {
        Variable::new(String::from(name))
    }
}

impl<T: FromBytes + ToBytes + CLTyped> Instance for Variable<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}
