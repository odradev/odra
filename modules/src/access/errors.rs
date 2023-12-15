use odra::OdraError;

#[derive(OdraError)]
pub enum Error {
    OwnerNotSet = 20_000,
    CallerNotTheOwner = 20_001,
    CallerNotTheNewOwner = 20_002,
    MissingRole = 20_003,
    RoleRenounceForAnotherAddress = 20_004
}
