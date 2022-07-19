use std::marker::PhantomData;

use odra_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLTyped, arithmetic::{OverflowingAdd, OverflowingSub},
};

use crate::{instance::Instance, UnwrapOrRevert};
use crate::ContractEnv;

#[derive(PartialEq, Debug)]
pub struct Variable<T> {
    name: String,
    ty: PhantomData<T>,
}

/// <3
impl<T: FromBytes + ToBytes + CLTyped + Default> Variable<T> {
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default> Variable<V> {
    pub fn add(&self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        ContractEnv::set_var(&self.name, new_value);
    }
}

impl<V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default> Variable<V> {
    pub fn subtract(&self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        ContractEnv::set_var(&self.name, new_value);
    }
}

impl<T: FromBytes + ToBytes + CLTyped> Variable<T> {
    pub fn new(name: String) -> Self {
        Variable {
            name,
            ty: PhantomData::<T>::default(),
        }
    }

    pub fn get(&self) -> Option<T> {
        match ContractEnv::get_var(&self.name) {
            Some(value) => Some(value.into_t::<T>().unwrap_or_revert()),
            None => None,
        }
    }

    pub fn set(&self, value: T) {
        ContractEnv::set_var(&self.name, value);
    }

    pub fn path(&self) -> &str {
        &self.name
    }
}

impl<T: FromBytes + ToBytes + CLTyped> From<&str> for Variable<T> {
    fn from(name: &str) -> Self {
        Variable::new(name.to_string())
    }
}

impl<T: FromBytes + ToBytes + CLTyped> Instance for Variable<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}
