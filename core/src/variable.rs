use core::marker::PhantomData;

use crate::{
    contract_env,
    instance::{DynamicInstance, StaticInstance},
    prelude::vec::Vec,
    types::OdraType,
    UnwrapOrRevert
};
use odra_types::arithmetic::{OverflowingAdd, OverflowingSub};

/// Data structure for storing a single value.
#[derive(Clone)]
pub struct Variable<T> {
    ty: PhantomData<T>,
    namespace_buffer: Vec<u8>
}

// <3
impl<T: OdraType + Default> Variable<T> {
    /// Reads from the storage, if theres no value in the storage the default value is returned.
    #[inline(always)]
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<V: OdraType + OverflowingAdd + Default> Variable<V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    #[inline(always)]
    pub fn add(&mut self, value: V) {
        let current_value = self.get_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        self.set(new_value);
    }
}

impl<V: OdraType + OverflowingSub + Default> Variable<V> {
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    #[inline(always)]
    pub fn subtract(&mut self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        self.set(new_value);
    }
}

impl<T: OdraType> Variable<T> {
    /// Reads from the storage or returns `None` or reverts something unexpected happens.
    #[inline(always)]
    pub fn get(&self) -> Option<T> {
        contract_env::get_var(&self.namespace_buffer)
    }

    /// Stores `value` to the storage.
    #[inline(always)]
    pub fn set(&mut self, value: T) {
        contract_env::set_var(&self.namespace_buffer, value);
    }
}

impl<T: OdraType> StaticInstance for Variable<T> {
    fn instance<'a>(keys: &'a [&'a str]) -> (Self, &'a [&'a str]) {
        let name = keys[0];
        (
            Variable {
                ty: PhantomData,
                namespace_buffer: name.as_bytes().to_vec()
            },
            &keys[1..]
        )
    }
}

impl<T: OdraType> DynamicInstance for Variable<T> {
    fn instance(namespace: &[u8]) -> Self {
        Variable {
            ty: PhantomData,
            namespace_buffer: namespace.to_vec()
        }
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::{test_env, StaticInstance, Variable};
    use odra_mock_vm::types::OdraType;
    use odra_types::arithmetic::ArithmeticsError;
    const SHARED_VALUE: [&str; 1] = ["shared_value"];

    #[test]
    fn test_get() {
        // Given uninitialized var.
        let value = 100;
        let mut var = Variable::<u8>::default();

        // When set a value.
        var.set(value);

        // The the value can be returned.
        assert_eq!(var.get().unwrap(), value);

        // When override.
        let value = 200;
        var.set(value);

        // Then the value is updated.
        assert_eq!(var.get().unwrap(), value);
    }

    #[test]
    fn get_default_value() {
        // Given uninitialized var.
        let var = Variable::<u8>::default();

        // Raw get returns None.
        assert_eq!(var.get(), None);
        // get_or_default returns the default value.
        assert_eq!(var.get_or_default(), 0);
    }

    #[test]
    fn test_add() {
        // Given var = u8::MAX-1.
        let initial_value = u8::MAX - 1;
        let mut var = Variable::<u8>::init(initial_value);

        // When add 1.
        var.add(1);

        // Then the value should be u8::MAX.
        assert_eq!(var.get_or_default(), initial_value + 1);

        // When add 1 to max value.
        // Then should revert.
        test_env::assert_exception(ArithmeticsError::AdditionOverflow, || var.add(1));
    }

    #[test]
    fn test_subtract() {
        // Given var = 2.
        let initial_value = 2;
        let mut var = Variable::<u8>::init(initial_value);
        // When subtract 1.
        var.subtract(1);

        // Then the value should be reduced by 1.
        assert_eq!(var.get_or_default(), initial_value - 1);

        // When subtraction causes overflow.
        // Then it reverts.
        test_env::assert_exception(ArithmeticsError::SubtractingOverflow, || var.subtract(2));
    }

    #[test]
    fn test_instances_with_the_same_namespace() {
        // Given two variables with the same namespace.
        let value = 42;
        let mut x = Variable::<u8>::instance(&SHARED_VALUE).0;
        let y = Variable::<u8>::instance(&SHARED_VALUE).0;

        // When set a value for the first variable.
        x.set(value);

        // Then both returns the same value.
        assert_eq!(y.get_or_default(), value);
        assert_eq!(x.get(), y.get());
    }

    impl<T: OdraType> Default for Variable<T> {
        fn default() -> Self {
            StaticInstance::instance(&["v"]).0
        }
    }

    impl<T: OdraType> Variable<T> {
        fn init(value: T) -> Self {
            let mut var = Self::default();
            var.set(value);
            var
        }
    }
}
