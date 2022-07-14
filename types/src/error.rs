#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OdraError {
    ExecutionError(String),
    VmError(VmError),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VmError {
    NoSuchMethod(String),
    InvalidContractAddress,
}
