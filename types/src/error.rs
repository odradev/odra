use casper_types::bytesrepr;

use crate::arithmetic::ArithmeticsError;

const MAX_USER_ERROR: u16 = 32767;
const USER_ERROR_TOO_HIGH: u16 = 32768;
const UNWRAP_ERROR: u16 = u16::MAX;

#[derive(Clone, Debug, PartialEq)]
pub enum OdraError {
    ExecutionError(ExecutionError),
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

#[derive(Clone, Debug)]
pub struct ExecutionError(pub u16, String);

impl ExecutionError {
    pub fn new(code: u16, msg: &str) -> Self {
        if code > MAX_USER_ERROR {
            Self(USER_ERROR_TOO_HIGH, String::from(msg))
        } else {
            Self(code, String::from(msg))
        }
    }

    pub fn unwrap_error() -> Self {
        Self::new(UNWRAP_ERROR, "Unwrap error")
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    NoSuchMethod(String),
    InvalidContractAddress,
    InvalidContext,
    Other(String),
    Panic,
}
