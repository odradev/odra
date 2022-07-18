use casper_types::bytesrepr;

#[derive(Clone, Debug)]
pub enum OdraError {
    ExecutionError(u32, String),
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
        let shift: u32 = u16::MAX as u32;
        OdraError::ExecutionError(code as u32 + shift, String::from(msg))
    }

    fn internal_execution_err(code: u16, msg: &str) -> Self {
        OdraError::ExecutionError(code as u32, String::from(msg))
    }
}

impl From<bytesrepr::Error> for OdraError {
    fn from(error: bytesrepr::Error) -> Self {
        match error {
            bytesrepr::Error::EarlyEndOfStream => OdraError::internal_execution_err(1, "EarlyEndOfStream"),
            bytesrepr::Error::Formatting => OdraError::internal_execution_err(2, "Formatting"),
            bytesrepr::Error::LeftOverBytes => OdraError::internal_execution_err(3, "LeftOverBytes"),
            bytesrepr::Error::OutOfMemory => OdraError::internal_execution_err(4, "OutOfMemory"),
            bytesrepr::Error::ExceededRecursionDepth => OdraError::internal_execution_err(5, "ExceededRecursionDepth"),
        }
    }
}

impl From<casper_types::CLValueError> for OdraError {
    fn from(error: casper_types::CLValueError) -> Self {
        match error {
            casper_types::CLValueError::Serialization(err) => err.into(),
            casper_types::CLValueError::Type(ty) => OdraError::internal_execution_err(5, &format!("Type mismatch {:?}", ty)),
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
    Panic
}
