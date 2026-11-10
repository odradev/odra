//! Dapp information modules.
//!
//! A dapp is usually made of several contracts. A [`DappRegistry`] groups them: it keeps the dapp's
//! [`DappMetadata`] and the list of contracts that belong to the dapp. A contract belongs to at most
//! one dapp: it implements [`DappContract`] and points to its registry, and the registry verifies that
//! before adding the contract. A contract registered as a registrar may add other contracts.
//!
//! The base modules, [`DappRegistryBase`] and [`DappContractBase`], do not check who calls them. Compose
//! them with an access module such as [`Ownable`](crate::access::Ownable) or
//! [`AccessControl`](crate::access::AccessControl), as [`OwnedDappRegistry`] does.
//!
//! The dapp data of a contract lives in one dictionary under the `__dapp` named key, every item a whole
//! value (`__dapp->registry`, `__dapp->name`, the `__dapp->contracts` list, ..., see [`storage`]), so it
//! can be read from the chain without calling the contracts. A contract composing [`DappRegistryBase`]
//! calls its `init`, which creates the dictionary with empty `contracts` and `registrars` lists, so a
//! registry is recognizable before any contract is added.
//!
//! A contract that creates other contracts of the dapp, e.g. a factory, is registered as a registrar.
//! It passes [`DappContractBase::register_address`] to each contract it creates, so that one points to
//! the registry too, and adds it with [`DappContractBase::register`].
pub mod errors;
pub mod events;
pub mod storage;

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
