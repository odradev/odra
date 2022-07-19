use casper_types::bytesrepr;

use crate::arithmetic::ArithmeticsError;

const MAX_USER_ERROR: u16 = 32767;
const USER_ERROR_TOO_HIGH: u16 = 32768;

#[derive(Clone, Debug)]
pub enum OdraError {
    ExecutionError(u16, String),
    VmError(VmError),
    Unknown,
}

impl PartialEq for OdraError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ExecutionError(l0, _), Self::ExecutionError(r0, _)) => l0 == r0,
            (Self::VmError(l0), Self::VmError(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl OdraError {
    pub fn execution_err(code: u16, msg: &str) -> Self {
        if code > MAX_USER_ERROR {
            OdraError::ExecutionError(USER_ERROR_TOO_HIGH, String::from(msg))
        } else {
            OdraError::ExecutionError(code, String::from(msg))
        }
    }

    fn internal_execution_err(code: u16, msg: &str) -> Self {
        OdraError::ExecutionError(code + USER_ERROR_TOO_HIGH, String::from(msg))
    }
}

impl From<bytesrepr::Error> for OdraError {
    fn from(error: bytesrepr::Error) -> Self {
        match error {
            bytesrepr::Error::EarlyEndOfStream => {
                OdraError::internal_execution_err(1, "EarlyEndOfStream")
            }
            bytesrepr::Error::Formatting => OdraError::internal_execution_err(2, "Formatting"),
            bytesrepr::Error::LeftOverBytes => {
                OdraError::internal_execution_err(3, "LeftOverBytes")
            }
            bytesrepr::Error::OutOfMemory => OdraError::internal_execution_err(4, "OutOfMemory"),
            bytesrepr::Error::ExceededRecursionDepth => {
                OdraError::internal_execution_err(5, "ExceededRecursionDepth")
            }
        }
    }
}

impl From<casper_types::CLValueError> for OdraError {
    fn from(error: casper_types::CLValueError) -> Self {
        match error {
            casper_types::CLValueError::Serialization(err) => err.into(),
            casper_types::CLValueError::Type(ty) => {
                OdraError::internal_execution_err(6, &format!("Type mismatch {:?}", ty))
            }
        }
    }
}

impl Into<OdraError> for ArithmeticsError {
    fn into(self) -> OdraError {
        match self {
            ArithmeticsError::AdditionOverflow => {
                OdraError::internal_execution_err(7, "Addition overflow")
            }
            ArithmeticsError::SubtractingOverflow => {
                OdraError::internal_execution_err(8, "Subtracting overflow")
            }
        }
    }
}

impl From<Box<dyn std::any::Any + Send>> for OdraError {
    fn from(_: Box<dyn std::any::Any + Send>) -> Self {
        OdraError::VmError(VmError::Panic)
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
