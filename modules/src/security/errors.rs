use odra::OdraError;

pub enum Error {
    PausedRequired = 21_000,
    UnpausedRequired = 21_001
}

impl From<Error> for OdraError {
    fn from(error: Error) -> Self {
        OdraError::user(error as u16)
    }
}
