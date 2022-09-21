use crate::bytesrepr;

use crate::arithmetic::ArithmeticsError;

const MAX_USER_ERROR: u16 = 32767;
const USER_ERROR_TOO_HIGH: u16 = 32768;
const UNWRAP_ERROR: u16 = u16::MAX;

/// General error type in Odra framework
#[derive(Clone, Debug, PartialEq)]
pub enum OdraError {
    /// An error that can occur during smart contract execution
    ExecutionError(ExecutionError),
    /// An internal virtual machine error
    VmError(VmError),
}

impl From<bytesrepr::Error> for ExecutionError {
    fn from(error: bytesrepr::Error) -> Self {
        match error {
            bytesrepr::Error::EarlyEndOfStream => ExecutionError::internal(1, "EarlyEndOfStream"),
            bytesrepr::Error::Formatting => ExecutionError::internal(2, "Formatting"),
            bytesrepr::Error::LeftOverBytes => ExecutionError::internal(3, "LeftOverBytes"),
            bytesrepr::Error::OutOfMemory => ExecutionError::internal(4, "OutOfMemory"),
            bytesrepr::Error::ExceededRecursionDepth => {
                ExecutionError::internal(5, "ExceededRecursionDepth")
            }
        }
    }
}

impl From<casper_types::CLValueError> for ExecutionError {
    fn from(error: casper_types::CLValueError) -> Self {
        match error {
            casper_types::CLValueError::Serialization(err) => err.into(),
            casper_types::CLValueError::Type(ty) => {
                ExecutionError::internal(6, &format!("Type mismatch {:?}", ty))
            }
        }
    }
}

impl From<ArithmeticsError> for ExecutionError {
    fn from(error: ArithmeticsError) -> Self {
        match error {
            ArithmeticsError::AdditionOverflow => Self::internal(7, "Addition overflow"),
            ArithmeticsError::SubtractingOverflow => Self::internal(8, "Subtracting overflow"),
        }
    }
}

impl From<Box<dyn std::any::Any + Send>> for OdraError {
    fn from(_: Box<dyn std::any::Any + Send>) -> Self {
        OdraError::VmError(VmError::Panic)
    }
}

/// An error that can occur during smart contract execution
///
/// It is represented by an error code and a human-readable message.
///
/// Errors codes 0..32767 are available for the user to define custom error
/// in smart contracts.
/// 32768 code is a special code representing a violation of the custom error code space.
///
/// The rest of codes 32769..[u16::MAX](u16::MAX), are used internally by the framework.
#[derive(Clone, Debug)]
pub struct ExecutionError(u16, String);

impl ExecutionError {
    /// Creates an instance with specified code and message.
    ///
    /// If the custom error code space is violated, an error with code 32768 is returned.
    pub fn new(code: u16, msg: &str) -> Self {
        if code > MAX_USER_ERROR {
            Self(
                USER_ERROR_TOO_HIGH,
                String::from("User error too high. The code should be in range 0..32767."),
            )
        } else {
            Self(code, String::from(msg))
        }
    }

    /// Creates a specific type of error, meaning that value unwrapping failed.
    pub fn unwrap_error() -> Self {
        Self::new(UNWRAP_ERROR, "Unwrap error")
    }

    /// Return the underlying error code
    pub fn code(&self) -> u16 {
        self.0
    }

    fn internal(code: u16, msg: &str) -> Self {
        ExecutionError(code + USER_ERROR_TOO_HIGH, String::from(msg))
    }
}

impl PartialEq for ExecutionError {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl From<ExecutionError> for OdraError {
    fn from(error: ExecutionError) -> Self {
        Self::ExecutionError(error)
    }
}

/// An internal virtual machine error
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    /// Non existing host entrypoint was called.
    NoSuchMethod(String),
    /// Accessing a contract with an invalid address.
    InvalidContractAddress,
    /// Error calling a host function in a wrong context.
    InvalidContext,
    /// Non-specified error with a custom message.
    Other(String),
    /// Unspecified error.
    Panic,
}

/// Error that can occur while operating on a collection.
pub enum CollectionError {
    // The requested index is bigger than the max collection index.
    IndexOutOfBounds,
}

impl From<CollectionError> for ExecutionError {
    fn from(error: CollectionError) -> Self {
        match error {
            CollectionError::IndexOutOfBounds => Self::internal(9, "Index out of bounds"),
        }
    }
}

impl From<CollectionError> for OdraError {
    fn from(error: CollectionError) -> Self {
        Into::<ExecutionError>::into(error).into()
    }
}
