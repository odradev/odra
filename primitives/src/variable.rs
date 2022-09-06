use std::marker::PhantomData;

use odra_env::unwrap_or_revert::UnwrapOrRevert;
use odra_env::ContractEnv;
use odra_types::{
    arithmetic::{OverflowingAdd, OverflowingSub},
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};

use crate::instance::Instance;
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
        Variable::new(name.to_string())
    }
}

impl<T: FromBytes + ToBytes + CLTyped> Instance for Variable<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use odra_env::TestEnv;
    use odra_types::{
        arithmetic::ArithmeticsError,
        bytesrepr::{FromBytes, ToBytes},
        CLTyped,
    };

    use crate::{instance::Instance, variable::Variable};

    #[test]
    fn test_get() {
        // Given uninitialized var.
        let value = 100;
        let var = Variable::<u8>::default();

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
        let var = Variable::<u8>::init(initial_value);

        // When add 1.
        var.add(1);

        // Then the value should be u8::MAX.
        assert_eq!(var.get_or_default(), initial_value + 1);

        // When add 1 to max value.
        // Then should revert.
        TestEnv::assert_exception(ArithmeticsError::AdditionOverflow, || {
            var.add(1);
        });
    }

    #[test]
    fn test_subtract() {
        // Given var = 2.
        let initial_value = 2;
        let var = Variable::<u8>::init(initial_value);
        // When subtract 1.
        var.subtract(1);

        // Then the value should be reduced by 1.
        assert_eq!(var.get_or_default(), initial_value - 1);

        // When subtraction causes overflow.
        // Then it reverts.
        TestEnv::assert_exception(ArithmeticsError::SubtractingOverflow, || {
            var.subtract(2);
        });
    }

    #[test]
    fn test_instances_with_the_same_namespace() {
        // Given two variables with the same namespace.
        let namespace = "shared_value";
        let value = 42;
        let x = Variable::<u8>::instance(namespace);
        let y = Variable::<u8>::instance(namespace);

        // When set a value for the first variable.
        x.set(value);

        // Then both returns the same value.
        assert_eq!(y.get_or_default(), value);
        assert_eq!(x.get(), y.get());
    }

    impl<T> Default for Variable<T>
    where
        T: FromBytes + ToBytes + CLTyped,
    {
        fn default() -> Self {
            Instance::instance("v")
        }
    }

    impl<T> Variable<T>
    where
        T: FromBytes + ToBytes + CLTyped,
    {
        fn init(value: T) -> Self {
            let var = Self::default();
            var.set(value);
            var
        }
    }
}
