use core::{fmt::Debug, marker::PhantomData};

use crate::{
    contract_env,
    instance::{DynamicInstance, StaticInstance},
    prelude::vec::Vec,
    UnwrapOrRevert
};
use odra_types::{arithmetic::{OverflowingAdd, OverflowingSub}, casper_types::bytesrepr::{ToBytes, FromBytes}};

/// Data structure for storing key-value pairs.
#[derive(Clone)]
pub struct Mapping<K, V> {
    key_ty: PhantomData<K>,
    value_ty: PhantomData<V>,
    namespace_buffer: Vec<u8>
}

impl<K: ToBytes, V: ToBytes + FromBytes> Mapping<K, V> {
    /// Reads `key` from the storage or returns `None`.
    #[inline(always)]
    pub fn get(&self, key: &K) -> Option<V> {
        contract_env::get_dict_value(&self.namespace_buffer, key)
    }

    /// Sets `value` under `key` to the storage. It overrides by default.
    #[inline(always)]
    pub fn set(&mut self, key: &K, value: V) {
        contract_env::set_dict_value(&self.namespace_buffer, key, value);
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + Default> Mapping<K, V> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, key: &K) -> V {
        self.get(key).unwrap_or_default()
    }
}

impl<K: ToBytes, V: DynamicInstance> Mapping<K, V> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_instance(&self, key: &K) -> V {
        let key_hash = key.to_bytes().unwrap_or_revert();

        let mut result = Vec::with_capacity(key_hash.len() + self.namespace_buffer.len());
        result.extend_from_slice(self.namespace_buffer.as_slice());
        result.extend_from_slice(key_hash.as_slice());
        V::instance(&result)
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + OverflowingAdd + Default> Mapping<K, V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        self.set(key, new_value);
    }
}

impl<K: ToBytes, V: ToBytes + FromBytes + OverflowingSub + Default + Debug + PartialOrd> Mapping<K, V> {
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&mut self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        self.set(key, new_value);
    }
}

impl<K: ToBytes, V> StaticInstance for Mapping<K, V> {
    fn instance<'a>(keys: &'a [&'a str]) -> (Self, &'a [&'a str]) {
        (
            Self {
                key_ty: PhantomData,
                value_ty: PhantomData,
                namespace_buffer: keys[0].as_bytes().to_vec()
            },
            &keys[1..]
        )
    }
}

impl<K: ToBytes, V> DynamicInstance for Mapping<K, V> {
    fn instance(namespace: &[u8]) -> Self {
        Self {
            key_ty: PhantomData,
            value_ty: PhantomData,
            namespace_buffer: namespace.to_vec()
        }
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::{instance::StaticInstance, mapping::Mapping, prelude::string::String, test_env};
    use odra_types::{arithmetic::ArithmeticsError, casper_types::bytesrepr::{FromBytes, ToBytes}};

    const SHARED_VALUE: [&str; 1] = ["shared_value"];

    #[test]
    fn test_get() {
        // Given uninitialized var.
        let value = 100;
        let key = String::from("k");
        let mut map = Mapping::<String, u8>::default();

        // When set a value.
        map.set(&key, value);

        // The the value can be returned.
        assert_eq!(map.get(&key).unwrap(), value);

        // When override.
        let value = 200;
        map.set(&key, value);

        // Then the value is updated
        assert_eq!(map.get(&key).unwrap(), value);
    }

    #[test]
    fn get_default_value() {
        // Given uninitialized var.
        let map = Mapping::<String, u8>::default();

        // Raw get returns None.
        let key = String::from("k");
        assert_eq!(map.get(&key), None);
        // get_or_default returns the default value.
        assert_eq!(map.get_or_default(&key), 0);
    }

    #[test]
    fn test_add() {
        // Given var = u8::MAX-1.
        let initial_value = u8::MAX - 1;
        let key = String::from("k");
        let mut map = Mapping::<String, u8>::init(&key, initial_value);

        // When add 1.
        map.add(&key, 1);

        // Then the value should be u8::MAX.
        assert_eq!(map.get_or_default(&key), initial_value + 1);

        // When add 1 to max value.
        // Then should revert.
        test_env::assert_exception(ArithmeticsError::AdditionOverflow, || {
            map.add(&key, 1);
        });
    }

    #[test]
    fn test_subtract() {
        // Given var = 2.
        let initial_value = 2;
        let key = String::from("k");
        let mut map = Mapping::<String, u8>::init(&key, initial_value);
        // When subtract 1
        map.subtract(&key, 1);

        // Then the value should be reduced by 1.
        assert_eq!(map.get_or_default(&key), initial_value - 1);

        // When subtraction causes overflow.
        // Then it reverts.
        test_env::assert_exception(ArithmeticsError::SubtractingOverflow, || {
            map.subtract(&key, 2);
        });
    }

    #[test]
    fn test_instances_with_the_same_namespace() {
        // Given two variables with the same namespace.
        let key = String::from("k");
        let value = 42;
        let mut x = Mapping::<String, u8>::instance(&SHARED_VALUE).0;
        let y = Mapping::<String, u8>::instance(&SHARED_VALUE).0;

        // When set a value for the first variable.
        x.set(&key, value);

        // Then both returns the same value.
        assert_eq!(y.get_or_default(&key), value);
        assert_eq!(x.get(&key), y.get(&key));
    }

    impl<K, V> Default for Mapping<K, V>
    where
        K: ToBytes,
        V: ToBytes + FromBytes
    {
        fn default() -> Self {
            StaticInstance::instance(&["m"]).0
        }
    }

    impl<K, V> Mapping<K, V>
    where
        K: ToBytes,
        V: ToBytes + FromBytes
    {
        pub fn init(key: &K, value: V) -> Self {
            let mut var = Self::default();
            var.set(key, value);
            var
        }
    }
}
