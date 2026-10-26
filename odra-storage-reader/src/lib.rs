//! Reads the storage of a deployed Odra contract from a frontend, without calling it.
//!
//! The reader does not talk to the node. It resolves a field path to the place the value is
//! stored, and decodes the value the frontend fetched (e.g. with `casper-js-sdk`):
//!
//! 1. `currentContractHash(pkg.rawJSON.stored_value)` - the contract to query, from the
//!    `queryLatestGlobalState(packageHash, [])` result.
//! 2. `locate("balances", ["account-hash-..."])` - a dictionary item or a named key.
//! 3. `decode(item.rawJSON.stored_value.CLValue, location)` - the value, formatted like `odra-cli`.
//!
//! The contract is described by its contract schema file, the `*_schema.json` `odra-build`
//! writes next to the wasm.

mod error;
mod reader;
mod wasm;

pub use error::Error;
pub use odra_schema::CONTRACT_SCHEMA_VERSION;
pub use reader::{current_contract_hash, Location, StorageReader};
