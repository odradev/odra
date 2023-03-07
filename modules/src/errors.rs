use odra::execution_error;

execution_error! {
    pub enum Error {
        OwnerNotSet => 30_000,
        CallerNotTheOwner => 30_001,
    }
}
