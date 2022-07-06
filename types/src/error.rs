#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OdraError {
    ExecutionError(String),
    Unknown,
}