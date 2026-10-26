//! Named-key storage of the dapp modules.
//!
//! The dapp data lives under Casper named keys and dictionaries, so explorers and SDKs read it by
//! name, the way they read the `name` or `balances` of a CEP-18 token. The names carry a `__dapp_`
//! prefix, like the `__events` keys of the Casper Event Standard, so a registry composed into a
//! token contract does not clash with the token's keys.
#![allow(missing_docs)]

use odra::named_keys::{base64_encoded_key_value_storage, key_value_storage, single_value_storage};
use odra::prelude::*;

/// The dapp registry a contract belongs to. A factory contract stores its registry under the same
/// named key, see `odra::factory_dapp`.
pub const DAPP_REGISTRY_KEY: &str = odra::factory_dapp::DAPP_REGISTRY_KEY;
/// The name of the dapp.
pub const DAPP_NAME_KEY: &str = "__dapp_name";
/// A short description of the dapp.
pub const DAPP_DESCRIPTION_KEY: &str = "__dapp_description";
/// The URL of the dapp's website.
pub const DAPP_WEBSITE_URL_KEY: &str = "__dapp_website_url";
/// The URL of the dapp's icon.
pub const DAPP_ICON_URL_KEY: &str = "__dapp_icon_url";
/// The number of registered contracts; the registry itself is not counted.
pub const DAPP_CONTRACTS_COUNT_KEY: &str = "__dapp_contracts_count";
/// The registered contracts by their decimal index, `"0"` to the count minus one. An entry at or
/// beyond the count is stale: a removed contract leaves one behind.
pub const DAPP_CONTRACTS_KEY: &str = "__dapp_contracts";
/// The index of a registered contract in `__dapp_contracts` plus one, keyed by the base64 encoded
/// address; zero or no entry means the contract is not registered.
pub const DAPP_CONTRACT_INDICES_KEY: &str = "__dapp_contract_indices";
/// Whether a registered contract is a factory, keyed by the base64 encoded address.
pub const DAPP_FACTORIES_KEY: &str = "__dapp_factories";

single_value_storage!(DappRegistryStorage, Address, DAPP_REGISTRY_KEY);
single_value_storage!(DappNameStorage, String, DAPP_NAME_KEY);
single_value_storage!(DappDescriptionStorage, String, DAPP_DESCRIPTION_KEY);
single_value_storage!(DappWebsiteUrlStorage, String, DAPP_WEBSITE_URL_KEY);
single_value_storage!(DappIconUrlStorage, String, DAPP_ICON_URL_KEY);
single_value_storage!(DappContractsCountStorage, u32, DAPP_CONTRACTS_COUNT_KEY);
key_value_storage!(DappContractsStorage, DAPP_CONTRACTS_KEY, Address);
base64_encoded_key_value_storage!(
    DappContractIndicesStorage,
    DAPP_CONTRACT_INDICES_KEY,
    Address,
    u32
);
base64_encoded_key_value_storage!(DappFactoriesStorage, DAPP_FACTORIES_KEY, Address, bool);
