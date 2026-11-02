//! A contract that belongs to a dapp.
use super::errors::Error;
use super::events::DappRegistryChanged;
use super::storage::DappRegistryStorage;
use super::DappRegistryContractRef;
use odra::prelude::*;
use odra::ContractRef;

/// The interface of a contract that belongs to a dapp.
///
/// A contract belongs to at most one dapp. Every contract added to a [`DappRegistry`](super::DappRegistry)
/// must expose `get_dapp_registry`, which the registry calls to verify the contract points back to it.
/// Adding a contract that lacks this entry point fails with a VM error.
#[odra::external_contract]
pub trait DappContract {
    /// Returns the dapp registry the contract belongs to.
    fn get_dapp_registry(&self) -> Address;
}

/// Stores the address of the dapp registry a contract belongs to, in the `registry` item of the `__dapp`
/// dictionary (see [`storage`](super::storage)).
///
/// The module does not check who calls it. Call [`DappContractBase::set_dapp_registry`] from the
/// composing contract's `init`, and guard any later change with an access module.
///
/// A contract the registry has registered as a registrar adds the contracts it creates with
/// [`register`](DappContractBase::register), after passing them [`register_address`](DappContractBase::register_address)
/// so they point to the same registry.
#[odra::module(events = [DappRegistryChanged], errors = Error)]
pub struct DappContractBase {
    registry: SubModule<DappRegistryStorage>
}

#[odra::module]
impl DappContractBase {
    /// Returns the dapp registry the contract belongs to.
    ///
    /// Reverts with [`Error::DappRegistryNotSet`] if the registry has not been set.
    pub fn get_dapp_registry(&self) -> Address {
        self.registry
            .get()
            .unwrap_or_revert_with(&self.env(), Error::DappRegistryNotSet)
    }
}

impl DappContractBase {
    /// Points the contract to the given dapp registry and emits [`DappRegistryChanged`].
    ///
    /// Reverts with [`Error::NotAContract`] if `registry` is not a contract.
    /// SECURITY: Do not expose this function publicly without proper access control.
    pub fn set_dapp_registry(&mut self, registry: &Address) {
        if !registry.is_contract() {
            self.env().revert(Error::NotAContract);
        }
        let previous_registry = self.registry.get();
        self.registry.set(*registry);
        self.env().emit_event(DappRegistryChanged {
            previous_registry,
            new_registry: *registry
        });
    }

    /// Returns the dapp registry the contract belongs to, to pass to a contract it creates.
    ///
    /// Reverts with [`Error::DappRegistryNotSet`] if the registry has not been set.
    pub fn register_address(&self) -> Address {
        self.get_dapp_registry()
    }

    /// Adds `dapp_contract` to the dapp registry the contract belongs to. The registry accepts it only
    /// from a registrar, and only if `dapp_contract` points to the registry.
    pub fn register(&self, dapp_contract: Address) {
        DappRegistryContractRef::new(self.env(), self.get_dapp_registry())
            .add_dapp_contract(&dapp_contract, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn register_not_set() {
        let env = odra_test::odra_env();
        let contract = DappContractBase::deploy(&env, NoArgs);
        assert_eq!(
            contract.try_get_dapp_registry(),
            Err(Error::DappRegistryNotSet.into())
        );
    }
}
