use odra_core::casper_event_standard;

/// The state key name.
pub const STATE_KEY: &str = "state";

/// Constuctor group name.
pub const CONSTRUCTOR_GROUP_NAME: &str = "constructor_group";
/// The key under which the events are stored.
pub const EVENTS: &str = casper_event_standard::EVENTS_DICT;

/// The key under which the events length is stored.
pub const EVENTS_LENGTH: &str = casper_event_standard::EVENTS_LENGTH;

/// The key under which the contract main purse URef is stored.
pub const CONTRACT_MAIN_PURSE: &str = "__contract_main_purse";

/// The key under which the contract cargo purse URef is stored.
pub const CONTRACT_CARGO_PURSE: &str = "__contract_cargo_purse";

/// The key under which the reentrancy guard status is stored.
pub const REENTRANCY_GUARD: [u8; 18] = *b"__reentrancy_guard";

/// The key for account's cargo purse.
pub const CARGO_PURSE_KEY: &str = "__cargo_purse";

/// The key for the result bytes. It's used in test_env.
pub const RESULT_KEY: &str = "__result";

/// The arg name of a temporally purse that is used transfer tokens to a contract.
pub const CARGO_PURSE_ARG: &str = "cargo_purse";

/// The arg name of the contract package hash.
pub const CONTRACT_PACKAGE_HASH_ARG: &str = "contract_package_hash";

/// The arg name of the entry point.
pub const ENTRY_POINT_ARG: &str = "entry_point";

/// The arg name of the args.
pub const ARGS_ARG: &str = "args";

/// The arg name of the CSPR attached amount.
pub const ATTACHED_VALUE_ARG: &str = "attached_value";

/// The arg name for `amount` argument.
pub const AMOUNT_ARG: &str = "amount";

/// Constructor name
pub const CONSTRUCTOR_NAME: &str = "init";
