use odra::OdraError;

#[derive(OdraError)]
pub enum Error {
    PausedRequired = 21_000,
    UnpausedRequired = 21_001
}
