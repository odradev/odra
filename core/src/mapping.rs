use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::instance::Instance;
use crate::types::OdraType;
use crate::{contract_env, UnwrapOrRevert};
use odra_types::arithmetic::{OverflowingAdd, OverflowingSub};

/// Data structure for storing key-value pairs.
#[derive(Debug)]
pub struct Mapping<K, V> {
    name: String,
    key_ty: PhantomData<K>,
    value_ty: PhantomData<V>
}

impl<K: OdraType + Hash, V: OdraType> Mapping<K, V> {
    /// Creates a new Mapping instance.
    pub fn new(name: String) -> Self {
        Mapping {
            name,
            key_ty: PhantomData::<K>::default(),
            value_ty: PhantomData::<V>::default()
        }
    }

    /// Reads `key` from the storage or returns `None`.
    pub fn get(&self, key: &K) -> Option<V> {
        contract_env::get_dict_value(&self.name, key)
    }

    /// Sets `value` under `key` to the storage. It overrides by default.
    pub fn set(&mut self, key: &K, value: V) {
        contract_env::set_dict_value(&self.name, key, value);
    }

    /// Return the named key path to the variable.
    pub fn path(&self) -> &str {
        &self.name
    }
}

impl<PK: OdraType + Hash, SK: OdraType + Hash, V: OdraType> Mapping<PK, Mapping<SK, V>> {
    /// Reads the `primary_key`'s nested `secondary_key` from the storage or returns `None`.
    ///
    /// If the nested mapping doesn't exist, `None` is returned.
    pub fn get_nested(&self, primary_key: &PK, secondary_key: &SK) -> Option<V> {
        match self.get(primary_key) {
            Some(mapping) => mapping.get(secondary_key),
            None => None
        }
    }

    /// Sets `value` in the `primary_key`'s nested mapping under `secondary_key` to the storage.
    /// It overrides by default.
    ///
    /// If the nested mapping doesn't exist, it will be created.
    pub fn set_nested(&mut self, primary_key: &PK, secondary_key: &SK, value: V) {
        let mut mapping = match self.get(primary_key) {
            Some(mapping) => mapping,
            None => {
                #[cfg(feature = "casper")]
                {
                    use odra_casper_backend::casper_contract::unwrap_or_revert::UnwrapOrRevert;

                    let preimage = secondary_key.to_bytes().unwrap_or_revert();
                    let bytes =
                        odra_casper_backend::casper_contract::contract_api::runtime::blake2b(
                            preimage
                        );
                    self.init_nested(bytes, primary_key);
                }
                #[cfg(feature = "mock-vm")]
                {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::Hasher;

                    let mut hasher = DefaultHasher::new();
                    hasher.write(secondary_key.ser().unwrap().as_slice());
                    let hash = hasher.finish().to_le_bytes();
                    self.init_nested(hash, primary_key);
                }
                self.get(primary_key).unwrap_or_revert()
            }
        };
        mapping.set(secondary_key, value);
    }

    fn init_nested<T: AsRef<[u8]>>(&mut self, data: T, primary_key: &PK) {
        let key_hash = hex::encode(data);
        let mapping = Mapping::new(format!("{}_{}", key_hash, self.name));
        self.set(primary_key, mapping);
    }
}

impl<K: OdraType + Hash, V: OdraType + Default> Mapping<K, V> {
    /// Reads `key` from the storage or the default value is returned.
    pub fn get_or_default(&self, key: &K) -> V {
        self.get(key).unwrap_or_default()
    }
}

impl<K: OdraType + Hash, V: OdraType + OverflowingAdd + Default> Mapping<K, V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        contract_env::set_dict_value(&self.name, key, new_value);
    }
}

impl<K: OdraType + Hash, V: OdraType + OverflowingSub + Default + Debug + PartialOrd>
    Mapping<K, V>
{
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&mut self, key: &K, value: V) {
        let current_value = self.get(key).unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        contract_env::set_dict_value(&self.name, key, new_value);
    }
}

impl<K: OdraType + Hash, V: OdraType> From<&str> for Mapping<K, V> {
    fn from(name: &str) -> Self {
        Mapping::new(name.to_string())
    }
}

impl<K: OdraType + Hash, V: OdraType> Instance for Mapping<K, V> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}

#[cfg(feature = "mock-vm")]
impl<K: OdraType + Hash, V: OdraType> crate::types::BorshDeserialize for Mapping<K, V> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self::new(crate::types::BorshDeserialize::deserialize(buf)?))
    }
}

#[cfg(feature = "mock-vm")]
impl<K: OdraType + Hash, V: OdraType> crate::types::BorshSerialize for Mapping<K, V> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        crate::types::BorshSerialize::serialize(&self.path(), writer)?;
        Ok(())
    }
}

#[cfg(feature = "casper")]
impl<K: OdraType + Hash, V: OdraType> crate::casper::casper_types::CLTyped for Mapping<K, V> {
    fn cl_type() -> crate::casper::casper_types::CLType {
        crate::casper::casper_types::CLType::Any
    }
}

#[cfg(feature = "casper")]
impl<K: OdraType + Hash, V: OdraType> crate::casper::casper_types::bytesrepr::ToBytes
    for Mapping<K, V>
{
    fn to_bytes(&self) -> Result<Vec<u8>, crate::casper::casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend("mapping".to_bytes()?);
        result.extend(&self.path().to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        "mapping".serialized_length() + self.path().serialized_length()
    }
}

#[cfg(feature = "casper")]
impl<K: OdraType + Hash, V: OdraType> crate::casper::casper_types::bytesrepr::FromBytes
    for Mapping<K, V>
{
    fn from_bytes(
        bytes: &[u8]
    ) -> Result<(Self, &[u8]), crate::casper::casper_types::bytesrepr::Error> {
        let (_, bytes): (String, _) =
            crate::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (name, bytes) = crate::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((Mapping::new(name), bytes))
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::{test_env, Instance, Mapping};
    use core::hash::Hash;
    use odra_mock_vm::types::OdraType;
    use odra_types::arithmetic::ArithmeticsError;

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
            let mut map = Mapping::<String, u8>::default();
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
            let mut map = Mapping::<String, u8>::default();
            map.subtract(&key, 2);
        });
    }

    #[test]
    fn test_instances_with_the_same_namespace() {
        // Given two variables with the same namespace.
        let namespace = "shared_value";
        let key = String::from("k");
        let value = 42;
        let mut x = Mapping::<String, u8>::instance(namespace);
        let y = Mapping::<String, u8>::instance(namespace);

        // When set a value for the first variable.
        x.set(&key, value);

        // Then both returns the same value.
        assert_eq!(y.get_or_default(&key), value);
        assert_eq!(x.get(&key), y.get(&key));
    }

    impl<K, V> Default for Mapping<K, V>
    where
        K: OdraType + Hash,
        V: OdraType
    {
        fn default() -> Self {
            Instance::instance("m")
        }
    }

    impl<K, V> Mapping<K, V>
    where
        K: OdraType + Hash,
        V: OdraType
    {
        pub fn init(key: &K, value: V) -> Self {
            let mut var = Self::default();
            var.set(key, value);
            var
        }
    }
}
