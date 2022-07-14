#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OdraError {
    ExecutionError(String),
    VmError(VmError),
    Unknown,
}

impl OdraError {
    pub fn execution_err(msg: &str) -> Self {
        OdraError::ExecutionError(String::from(msg))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VmError {
    NoSuchMethod(String),
    InvalidContractAddress,
    Panic
}
