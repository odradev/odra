use std::marker::PhantomData;

use crate::types::OdraType;
use odra_types::arithmetic::{OverflowingAdd, OverflowingSub};

use crate::{contract_env, instance::Instance, UnwrapOrRevert};

/// Data structure for storing a single value.
#[derive(PartialEq, Eq, Debug)]
pub struct Variable<T> {
    name: String,
    ty: PhantomData<T>
}

// <3
impl<T: OdraType + Default> Variable<T> {
    /// Reads from the storage, if theres no value in the storage the default value is returned.
    pub fn get_or_default(&self) -> T {
        self.get().unwrap_or_default()
    }
}

impl<V: OdraType + OverflowingAdd + Default> Variable<V> {
    /// Utility function that gets the current value and adds the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn add(&mut self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_add(value).unwrap_or_revert();
        contract_env::set_var(&self.name, new_value);
    }
}

impl<V: OdraType + OverflowingSub + Default> Variable<V> {
    /// Utility function that gets the current value and subtracts the passed `value`
    /// and sets the new value to the storage.
    ///
    /// If the operation fails due to overflow, the currently executing contract reverts.
    pub fn subtract(&mut self, value: V) {
        let current_value = self.get().unwrap_or_default();
        let new_value = current_value.overflowing_sub(value).unwrap_or_revert();
        contract_env::set_var(&self.name, new_value);
    }
}

impl<T: OdraType> Variable<T> {
    /// Creates a new Variable instance.
    pub fn new(name: String) -> Self {
        Variable {
            name,
            ty: PhantomData::<T>::default()
        }
    }

    /// Reads from the storage or returns `None` or reverts something unexpected happens.
    pub fn get(&self) -> Option<T> {
        contract_env::get_var(&self.name)
    }

    /// Stores `value` to the storage.
    pub fn set(&mut self, value: T) {
        contract_env::set_var(&self.name, value);
    }

    /// Return the named key path to the variable.
    pub fn path(&self) -> &str {
        &self.name
    }
}

impl<T: OdraType> From<&str> for Variable<T> {
    fn from(name: &str) -> Self {
        Variable::new(name.to_string())
    }
}

impl<T: OdraType> Instance for Variable<T> {
    fn instance(namespace: &str) -> Self {
        namespace.into()
    }
}

#[cfg(feature = "mock-vm")]
impl<V: OdraType> crate::types::BorshDeserialize for Variable<V> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self::new(crate::types::BorshDeserialize::deserialize(buf)?))
    }
}

#[cfg(feature = "mock-vm")]
impl<V: OdraType> crate::types::BorshSerialize for Variable<V> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        crate::types::BorshSerialize::serialize(&self.path(), writer)?;
        Ok(())
    }
}

#[cfg(feature = "casper")]
impl<V: OdraType> crate::casper::casper_types::CLTyped for Variable<V> {
    fn cl_type() -> crate::casper::casper_types::CLType {
        crate::casper::casper_types::CLType::Any
    }
}

#[cfg(feature = "casper")]
impl<V: OdraType> crate::casper::casper_types::bytesrepr::ToBytes for Variable<V> {
    fn to_bytes(&self) -> Result<Vec<u8>, crate::casper::casper_types::bytesrepr::Error> {
        let mut result = Vec::with_capacity(self.serialized_length());
        result.extend("var".to_bytes()?);
        result.extend(&self.path().to_bytes()?);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        "var".serialized_length() + self.path().serialized_length()
    }
}

#[cfg(feature = "casper")]
impl<V: OdraType> crate::casper::casper_types::bytesrepr::FromBytes for Variable<V> {
    fn from_bytes(
        bytes: &[u8]
    ) -> Result<(Self, &[u8]), crate::casper::casper_types::bytesrepr::Error> {
        let (_, bytes): (String, _) =
            crate::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        let (name, bytes) = crate::casper::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
        Ok((Variable::new(name), bytes))
    }
}

#[cfg(all(feature = "mock-vm", test))]
mod tests {
    use crate::{test_env, Instance, Variable};
    use odra_mock_vm::types::OdraType;
    use odra_types::arithmetic::ArithmeticsError;

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
        test_env::assert_exception(ArithmeticsError::AdditionOverflow, || {
            let mut var = Variable::<u8>::default();
            var.add(1);
        });
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
        test_env::assert_exception(ArithmeticsError::SubtractingOverflow, || {
            let mut var = Variable::<u8>::default();
            var.subtract(2);
        });
    }

    #[test]
    fn test_instances_with_the_same_namespace() {
        // Given two variables with the same namespace.
        let namespace = "shared_value";
        let value = 42;
        let mut x = Variable::<u8>::instance(namespace);
        let y = Variable::<u8>::instance(namespace);

        // When set a value for the first variable.
        x.set(value);

        // Then both returns the same value.
        assert_eq!(y.get_or_default(), value);
        assert_eq!(x.get(), y.get());
    }

    impl<T: OdraType> Default for Variable<T> {
        fn default() -> Self {
            Instance::instance("v")
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
