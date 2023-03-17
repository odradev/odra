//! Constants utilizes by the [test_env](../odra_casper_test_env/index.html) and
//! [contract_env](../odra_casper_backend/contract_env/index.html).

/// The key under which the events are stored.
pub const EVENTS: &str = "__events";
/// The key under which the events length is stored.
pub const EVENTS_LENGTH: &str = "__events_length";
/// The key under which the contract main purse URef is stored.
pub const CONTRACT_MAIN_PURSE: &str = "__contract_main_purse";
/// The arg name of a temporally purse that is used transfer tokens to a contract.
pub const CARGO_PURSE_ARG: &str = "cargo_purse";
/// The key under which the reentrancy guard status is stored.
pub const REENTRANCY_GUARD: &str = "__reentrancy_guard";
