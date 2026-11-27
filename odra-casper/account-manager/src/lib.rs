#![doc = "Changes a Casper account's associated keys and action thresholds (multisig)."]
#![doc = ""]
#![doc = "The crate has three parts:"]
#![doc = "- [`ACCOUNT_MANAGER_WASM`], session code that applies an [`AccountChange`] in one"]
#![doc = "  transaction, and the argument names and [`Error`] codes it shares with it;"]
#![doc = "- [`AccountChange`] and [`AccountState`], the change and the configuration it applies to;"]
#![doc = "- [`plan`], which checks a change before it is sent and returns the configuration after it"]
#![doc = "  with [`Warning`]s, or the [`ValidationError`] the node would reject it with."]
#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod change;
mod error;
mod plan;

pub use change::{AccountChange, AccountState};
pub use error::Error;
pub use plan::{plan, Plan, ThresholdKind, ValidationError, Warning};

/// Session code that applies an [`AccountChange`] to the account that sends it.
///
/// It installs nothing, so it is sent as a session with `is_install_upgrade = false`. The
/// signers' weights must reach the account's current `key_management` threshold.
#[cfg(feature = "wasm")]
pub const ACCOUNT_MANAGER_WASM: &[u8] = include_bytes!("../resources/account_manager.wasm");

/// The keys to add, or whose weight to update: `Vec<(AccountHash, u8)>`.
pub const SET_KEYS_ARG: &str = "set_keys";
/// The keys to remove: `Vec<AccountHash>`.
pub const REMOVE_KEYS_ARG: &str = "remove_keys";
/// The new `deployment` threshold: `Option<u8>`.
pub const DEPLOYMENT_THRESHOLD_ARG: &str = "deployment_threshold";
/// The new `key_management` threshold: `Option<u8>`.
pub const KEY_MANAGEMENT_THRESHOLD_ARG: &str = "key_management_threshold";

/// The most associated keys an account can have (`max_associated_keys` in the chainspec of
/// Casper 2.x mainnet and testnet).
pub const MAX_ASSOCIATED_KEYS: usize = 100;
