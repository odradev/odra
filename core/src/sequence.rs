use crate::module::Revertible;
use crate::prelude::*;
use crate::{
    casper_types::{
        bytesrepr::{FromBytes, ToBytes},
        CLTyped
    },
    module::{ModuleComponent, ModulePrimitive},
    ContractEnv
};
use num_traits::{Num, One, Zero};

/// A module that stores a single value in the storage that can be read or incremented.
pub struct Sequence<T>
where
    T: Num + One + ToBytes + FromBytes + CLTyped
{
    env: Rc<ContractEnv>,
    index: u8,
    value: Var<T>
}

impl<T> Revertible for Sequence<T>
where
    T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped
{
    fn revert<E: Into<OdraError>>(&self, e: E) -> ! {
        self.env.revert(e)
    }
}

impl<T> Sequence<T>
where
    T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped
{
    /// Returns the current value of the sequence.
    pub fn get_current_value(&self) -> T {
        self.value.get().unwrap_or_default()
    }

    /// Increments the value of the sequence and returns the new value.
    pub fn next_value(&mut self) -> T {
        match self.value.get() {
            None => {
                self.value.set(T::zero());
                T::zero()
            }
            Some(value) => {
                let next = value + T::one();
                self.value.set(next);
                next
            }
        }
    }
}

impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> Sequence<T> {
    /// Returns the ContractEnv.
    pub fn env(&self) -> ContractEnv {
        self.env.child(self.index)
    }
}

/// Implements the `ModuleComponent` trait for the `Sequence` struct.
impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> ModuleComponent
    for Sequence<T>
{
    /// Creates a new instance of `Sequence` with the given environment and index.
    fn instance(env: Rc<ContractEnv>, index: u8) -> Self {
        Self {
            env: env.clone(),
            index,
            value: Var::instance(env.child(index).into(), 0)
        }
    }
}

impl<T: Num + One + Zero + Default + Copy + ToBytes + FromBytes + CLTyped> ModulePrimitive
    for Sequence<T>
{
}
