//! A contract that recognizes the dapps it belongs to.
use super::errors::Error;
use super::events::{DappRegistryForgotten, DappRegistryRecognized};
use super::storage::DappRegistriesStorage;
use odra::prelude::*;

/// The optional interface of a contract that says which dapps it belongs to.
///
/// A [`DappRegistry`](super::DappRegistry) decides on its own which contracts belong to the dapp: it does
/// not call its contracts, so any contract can be added, including one deployed before the registry, and
/// a contract can belong to several dapps. A contract implementing this interface confirms the
/// membership from its side, which users and explorers can check off-chain against the registry's list.
#[odra::external_contract]
pub trait DappMember {
    /// Returns true if the contract recognizes the dapp registry, that is, considers itself part of
    /// that dapp.
    fn recognizes_dapp_registry(&self, registry: &Address) -> bool;
}

/// Keeps the dapp registries a contract recognizes, implementing [`DappMember`].
///
/// The registries live in the `registries` list of the `__dapp` dictionary (see
/// [`storage`](super::storage)), so they can also be read from the chain without calling the contract.
///
/// The module does not check who calls it. Call [`init`](Self::init) from the composing contract's
/// `init` (or from its `upgrade`, for a contract deployed before), and guard
/// [`recognize_dapp_registry`](Self::recognize_dapp_registry) and
/// [`forget_dapp_registry`](Self::forget_dapp_registry) with an access module.
#[odra::module(events = [DappRegistryRecognized, DappRegistryForgotten], errors = Error)]
pub struct DappMemberBase {
    registries: SubModule<DappRegistriesStorage>
}

#[odra::module]
impl DappMemberBase {
    /// Creates the `__dapp` dictionary holding the recognized registries.
    pub fn init(&mut self) {
        self.registries.init();
    }

    /// Returns true if the contract recognizes the dapp registry.
    pub fn recognizes_dapp_registry(&self, registry: &Address) -> bool {
        self.registries().contains(registry)
    }
}

impl DappMemberBase {
    /// Recognizes the dapp registry and emits [`DappRegistryRecognized`]. Does nothing if the registry is
    /// already recognized.
    ///
    /// Reverts with [`Error::NotAContract`] if `registry` is not a contract.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn recognize_dapp_registry(&mut self, registry: &Address) {
        if !registry.is_contract() {
            self.env().revert(Error::NotAContract);
        }
        if self.recognizes_dapp_registry(registry) {
            return;
        }
        let mut registries = self.registries();
        registries.push(*registry);
        self.registries.set(registries);
        self.env().emit_event(DappRegistryRecognized {
            registry: *registry
        });
    }

    /// Stops recognizing the dapp registry and emits [`DappRegistryForgotten`]. Does nothing if the
    /// registry is not recognized.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn forget_dapp_registry(&mut self, registry: &Address) {
        if !self.recognizes_dapp_registry(registry) {
            return;
        }
        let mut registries = self.registries();
        registries.retain(|r| r != registry);
        self.registries.set(registries);
        self.env().emit_event(DappRegistryForgotten {
            registry: *registry
        });
    }

    fn registries(&self) -> Vec<Address> {
        self.registries.get().unwrap_or_default()
    }
}
