use odra::execution_error;

execution_error! {
    pub enum Error {
        PausedRequired => 21_000,
        UnpausedRequired => 21_001,
    }
}
