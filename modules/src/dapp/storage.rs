//! Named-key storage of the dapp modules.
//!
//! The dapp data of a contract lives in one Casper dictionary under the `__dapp` named key, so
//! explorers and SDKs read it by name, the way they read the `name` or `balances` of a CEP-18 token,
//! and it stays apart from the contract's own keys (like the `__events` keys of the Casper Event
//! Standard). Every item holds a whole value: a string, an address, or a list of addresses, e.g.
//! `__dapp->contracts` is the list of the registered contracts.
#![allow(missing_docs)]

use odra::named_keys::dictionary_value_storage;
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
/// The registered contracts, in the order they were added; the registry itself is not listed.
pub const CONTRACTS_KEY: &str = "contracts";
/// The registered contracts that may add other contracts.
pub const REGISTRARS_KEY: &str = "registrars";
/// The dapp registries a [`DappMember`](super::DappMember) contract recognizes.
pub const REGISTRIES_KEY: &str = "registries";

dictionary_value_storage!(DappNameStorage, String, DAPP_KEY, NAME_KEY);
dictionary_value_storage!(DappDescriptionStorage, String, DAPP_KEY, DESCRIPTION_KEY);
dictionary_value_storage!(DappWebsiteUrlStorage, String, DAPP_KEY, WEBSITE_URL_KEY);
dictionary_value_storage!(DappIconUrlStorage, String, DAPP_KEY, ICON_URL_KEY);
dictionary_value_storage!(DappContractsStorage, Vec<Address>, DAPP_KEY, CONTRACTS_KEY);
dictionary_value_storage!(
    DappRegistrarsStorage,
    Vec<Address>,
    DAPP_KEY,
    REGISTRARS_KEY
);
dictionary_value_storage!(
    DappRegistriesStorage,
    Vec<Address>,
    DAPP_KEY,
    REGISTRIES_KEY
);
