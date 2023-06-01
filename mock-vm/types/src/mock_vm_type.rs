/// MockVM Types implementation.
///
/// It supports the following types:
/// - Rust simple types: bool, u8, u32, u64, i32, i64, (), String.
/// - Larger unsigned integers: U256, U512.
/// - Rust complex types: Vec<T>, Result<T, E>, Option<T>.
use borsh::{BorshDeserialize, BorshSerialize};
use odra_types::{OdraError, VmError};

#[doc(hidden)]
pub trait MockSerializable {
    fn ser(&self) -> Result<Vec<u8>, MockVMSerializationError>;
}

#[doc(hidden)]
pub trait MockDeserializable: Sized {
    fn deser(bytes: Vec<u8>) -> Result<Self, MockVMSerializationError>;
}

impl<T: BorshSerialize> MockSerializable for T {
    fn ser(&self) -> Result<Vec<u8>, MockVMSerializationError> {
        borsh::to_vec(self).map_err(|_| MockVMSerializationError::SerializationError)
    }
}

impl<T: BorshDeserialize> MockDeserializable for T {
    fn deser(bytes: Vec<u8>) -> Result<Self, MockVMSerializationError> {
        <T as borsh::BorshDeserialize>::try_from_slice(&bytes)
            .map_err(|_| MockVMSerializationError::DeserializationError)
    }
}

/// An error that may occur while data de(serialization).
#[derive(Debug, PartialEq, Eq)]
pub enum MockVMSerializationError {
    /// Occurs if something went wrong during deserialization, eg. unexpected data format.
    DeserializationError,
    /// Occurs if something went wrong during data serialization.
    SerializationError
}

impl From<MockVMSerializationError> for OdraError {
    fn from(error: MockVMSerializationError) -> Self {
        match error {
            MockVMSerializationError::DeserializationError => {
                OdraError::VmError(VmError::Deserialization)
            }
            MockVMSerializationError::SerializationError => {
                OdraError::VmError(VmError::Serialization)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Address, U256, U512};

    use super::{MockDeserializable, MockSerializable};

    pub fn ser_deser<T: MockSerializable + MockDeserializable + PartialEq + std::fmt::Debug>(
        value: T
    ) {
        let bytes = value.ser().unwrap();
        let deserialized = T::deser(bytes).unwrap();
        assert_eq!(deserialized, value);
    }

    #[test]
    fn test_unit() {
        ser_deser(());
    }

    #[test]
    fn test_bool() {
        ser_deser(true);
        ser_deser(false);
    }

    #[test]
    fn test_basic_ints() {
        ser_deser(0u8);
        ser_deser(255u8);
        ser_deser(0u32);
        ser_deser(255u32);
        ser_deser(0u64);
        ser_deser(255u64);
        ser_deser(0i32);
        ser_deser(255i32);
        ser_deser(0i64);
        ser_deser(255i64);
    }

    #[test]
    fn test_large_ints() {
        ser_deser(U256::from(0));
        ser_deser(U256::from(255));
        ser_deser(U512::from(0));
        ser_deser(U512::from(255));
    }

    #[test]
    fn test_string() {
        ser_deser(String::from("hello world"));
    }

    #[test]
    fn test_vec() {
        ser_deser(Vec::<bool>::new());
        ser_deser(vec![vec![1u8], vec![1, 2]]);
    }

    #[test]
    fn test_result() {
        ser_deser(Result::<(), String>::Ok(()));
        ser_deser(Result::<(), String>::Err(String::from("Error")));
    }

    #[test]
    fn test_option() {
        ser_deser(Option::<u8>::None);
        ser_deser(Option::<u8>::Some(8));
    }

    #[test]
    fn test_tuples() {
        ser_deser((1u8, true));
        ser_deser((10u32, false, Option::<()>::None));
    }

    #[test]
    fn test_address() {
        ser_deser(Address::try_from(b"Satoshi").unwrap());
    }

    #[test]
    fn test_complex_example() {
        ser_deser((
            vec![0u8, 1u8],
            vec![vec![0u8, 1u8], vec![2u8, 3u8]],
            (Result::<Vec<u8>, ()>::Ok(vec![]), Option::<String>::None)
        ));
    }
}
