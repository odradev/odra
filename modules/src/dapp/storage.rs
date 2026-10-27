//! Named-key storage of the dapp modules.
//!
//! The dapp data of a contract lives in one Casper dictionary under the `__dapp` named key, so
//! explorers and SDKs read it by name, the way they read the `name` or `balances` of a CEP-18 token,
//! and it stays apart from the contract's own keys (like the `__events` keys of the Casper Event
//! Standard). The dictionary holds single values under fixed items (`name`, `registry`, ...) and maps
//! under items starting with a prefix (`registrars-><base64 encoded address>`, ...).
#![allow(missing_docs)]

use odra::named_keys::{
    base64_encoded_prefixed_key_value_storage, dictionary_value_storage, prefixed_key_value_storage
};
use odra::prelude::*;

/// The named key of the dictionary holding the dapp data.
pub const DAPP_KEY: &str = odra::factory_dapp::DAPP_KEY;
/// The dapp registry a factory contract adds its children to, see `odra::factory_dapp`.
pub const REGISTRY_KEY: &str = odra::factory_dapp::REGISTRY_KEY;
/// The name of the dapp.
pub const NAME_KEY: &str = "name";
/// A short description of the dapp.
pub const DESCRIPTION_KEY: &str = "description";
/// The URL of the dapp's website.
pub const WEBSITE_URL_KEY: &str = "website_url";
/// The URL of the dapp's icon.
pub const ICON_URL_KEY: &str = "icon_url";
/// The number of registered contracts; the registry itself is not counted.
pub const CONTRACTS_COUNT_KEY: &str = "contracts_count";
/// The registered contracts by their decimal index, `contracts->0` to the count minus one. An entry at
/// or beyond the count is stale: a removed contract leaves one behind.
pub const CONTRACTS_PREFIX: &str = "contracts->";
/// The index of a registered contract in the contracts plus one, by the base64 encoded address; zero or
/// no entry means the contract is not registered.
pub const CONTRACT_INDICES_PREFIX: &str = "contract_indices->";
/// Whether a registered contract may add other contracts, by the base64 encoded address.
pub const REGISTRARS_PREFIX: &str = "registrars->";
/// Whether a [`DappMember`](super::DappMember) contract recognizes a dapp registry, by the base64
/// encoded address of the registry.
pub const REGISTRIES_PREFIX: &str = "registries->";

dictionary_value_storage!(DappNameStorage, String, DAPP_KEY, NAME_KEY);
dictionary_value_storage!(DappDescriptionStorage, String, DAPP_KEY, DESCRIPTION_KEY);
dictionary_value_storage!(DappWebsiteUrlStorage, String, DAPP_KEY, WEBSITE_URL_KEY);
dictionary_value_storage!(DappIconUrlStorage, String, DAPP_KEY, ICON_URL_KEY);
dictionary_value_storage!(
    DappContractsCountStorage,
    u32,
    DAPP_KEY,
    CONTRACTS_COUNT_KEY
);
prefixed_key_value_storage!(DappContractsStorage, DAPP_KEY, CONTRACTS_PREFIX, Address);
base64_encoded_prefixed_key_value_storage!(
    DappContractIndicesStorage,
    DAPP_KEY,
    CONTRACT_INDICES_PREFIX,
    Address,
    u32
);
base64_encoded_prefixed_key_value_storage!(
    DappRegistrarsStorage,
    DAPP_KEY,
    REGISTRARS_PREFIX,
    Address,
    bool
);
base64_encoded_prefixed_key_value_storage!(
    DappRegistriesStorage,
    DAPP_KEY,
    REGISTRIES_PREFIX,
    Address,
    bool
);
