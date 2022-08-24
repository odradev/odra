use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::ContractEnv;
use crate::UnwrapOrRevert;
use odra_types::{
    arithmetic::{OverflowingAdd, OverflowingSub},
    bytesrepr::{FromBytes, ToBytes},
    CLTyped,
};

use crate::instance::Instance;

/// Data structure for storing key-value pairs.
#[derive(Debug)]
pub struct Mapping<K, V> {
    name: String,
    key_ty: PhantomData<K>,
    value_ty: PhantomData<V>,
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Mapping<K, V> {
    /// Creates a new Mapping instance.
    pub fn new(name: String) -> Self {
        Mapping {
            name,
            key_ty: PhantomData::<K>::default(),
            value_ty: PhantomData::<V>::default(),
        }
    }

    /// Reads `key` from the storage or returns `None`.
    pub fn get(&self, key: &K) -> Option<V> {
        let result = ContractEnv::get_dict_value(&self.name, key);
        result.map(|value| value.into_t::<V>().unwrap_or_revert())
    }

    /// Sets `value` under `key` to the storage. It overrides by default.
    pub fn set(&self, key: &K, value: V) {
        ContractEnv::set_dict_value(&self.name, key, value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + Default> Mapping<K, V> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, key: &K) -> V {
        self.get(key).unwrap_or_default()
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped + OverflowingAdd + Default>
    Mapping<K, V>
{
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        ContractEnv::set_dict_value(&self.name, key, new_value);
    }
}

impl<
        K: ToBytes + CLTyped + Hash,
        V: ToBytes + FromBytes + CLTyped + OverflowingSub + Default + Debug + PartialOrd,
    > Mapping<K, V>
{
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        ContractEnv::set_dict_value(&self.name, key, new_value);
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> From<&str> for Mapping<K, V> {
    fn from(name: &str) -> Self {
        Mapping::new(name.to_string())
    }
}

impl<K: ToBytes + CLTyped + Hash, V: ToBytes + FromBytes + CLTyped> Instance for Mapping<K, V> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::{Instance, Mapping};
    use core::hash::Hash;
    use odra_mock_vm::TestEnv;
    use odra_types::{
        arithmetic::ArithmeticsError,
        bytesrepr::{FromBytes, ToBytes},
        CLTyped, ExecutionError,
    };

    #[test]
    fn test_get() {
        // Given uninitialized var.
        let value = 100;
        let key = String::from("k");
        let var = Mapping::<String, u8>::default();

        // When set a value.
        var.set(&key, value);

        // The the value can be returned.
        assert_eq!(var.get(&key).unwrap(), value);

        // When override.
        let value = 200;
        var.set(&key, value);

        // Then the value is updated
        assert_eq!(var.get(&key).unwrap(), value);
    }

    #[test]
    fn get_default_value() {
        // Given uninitialized var.
        let var = Mapping::<String, u8>::default();

        // Raw get returns None.
        let key = String::from("k");
        assert_eq!(var.get(&key), None);
        // get_or_default returns the default value.
        assert_eq!(var.get_or_default(&key), 0);
    }

    #[test]
    fn test_add() {
        // Given var = u8::MAX-1.
        let initial_value = u8::MAX - 1;
        let key = String::from("k");
        let var = Mapping::<String, u8>::init(&key, initial_value);

        // When add 1.
        var.add(&key, 1);

        // Then the value should be u8::MAX.
        assert_eq!(var.get_or_default(&key), initial_value + 1);

        // When add 1 to max value.
        // Then should revert.
        TestEnv::assert_exception(
            Into::<ExecutionError>::into(ArithmeticsError::AdditionOverflow),
            || {
                var.add(&key, 1);
            },
        );
    }

    #[test]
    fn test_subtract() {
        // Given var = 2.
        let initial_value = 2;
        let key = String::from("k");
        let var = Mapping::<String, u8>::init(&key, initial_value);
        // When subtract 1
        var.subtract(&key, 1);

        // Then the value should be reduced by 1.
        assert_eq!(var.get_or_default(&key), initial_value - 1);

        // When subtraction causes overflow.
        // Then it reverts.
        TestEnv::assert_exception(
            Into::<ExecutionError>::into(ArithmeticsError::SubtractingOverflow),
            || {
                var.subtract(&key, 2);
            },
        );
    }

    impl<K, V> Default for Mapping<K, V>
    where
        K: ToBytes + CLTyped + Hash,
        V: ToBytes + FromBytes + CLTyped,
    {
        fn default() -> Self {
            Instance::instance("m")
        }
    }

    impl<K, V> Mapping<K, V>
    where
        K: ToBytes + CLTyped + Hash,
        V: ToBytes + FromBytes + CLTyped,
    {
        pub fn init(key: &K, value: V) -> Self {
            let var: Self = Default::default();
            var.set(key, value);
            var
        }
    }
}
