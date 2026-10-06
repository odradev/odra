//! Dapp information modules.
//!
//! A dapp is usually made of several contracts. A [`DappRegistry`] groups them: it keeps the dapp's
//! [`DappMetadata`] and the list of contracts that belong to the dapp. Every member contract implements
//! [`DappContract`] and points back to its registry, so the registry can verify each contract it adds.
//!
//! The base modules, [`DappRegistryBase`] and [`DappContractBase`], do not check who calls them. Compose
//! them with an access module such as [`Ownable`](crate::access::Ownable) or
//! [`AccessControl`](crate::access::AccessControl), as [`OwnedDappRegistry`] does.
pub mod errors;
pub mod events;

mod dapp_contract;
mod dapp_registry;
mod metadata;
mod owned_dapp_registry;
#[allow(missing_docs, dead_code)]
pub(crate) mod utils;

pub use dapp_contract::*;
pub use dapp_registry::*;
pub use metadata::DappMetadata;
pub use owned_dapp_registry::*;
