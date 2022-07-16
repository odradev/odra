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
        OdraError::ExecutionError(code, String::from(msg))
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
