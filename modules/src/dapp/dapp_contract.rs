//! A contract that belongs to a dapp.
use super::errors::Error;
use super::events::DappRegistryChanged;
use odra::prelude::*;

/// The interface of a contract that belongs to a dapp.
///
/// Every contract added to a [`DappRegistry`](super::DappRegistry) must expose `get_dapp_registry`,
/// which the registry calls to verify the contract points back to it. Adding a contract that lacks this
/// entry point fails with a VM error.
#[odra::external_contract]
pub trait DappContract {
    /// Points the contract to the given dapp registry.
    fn set_dapp_registry(&mut self, registry: &Address);
    /// Returns the dapp registry the contract belongs to.
    fn get_dapp_registry(&self) -> Address;
}

/// Stores the address of the dapp registry a contract belongs to.
///
/// The module does not check who calls it. Call [`DappContractBase::set_dapp_registry`] from the
/// composing contract's `init`, and guard any later change with an access module.
#[odra::module(events = [DappRegistryChanged], errors = Error)]
pub struct DappContractBase {
    registry: Var<Address>
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
